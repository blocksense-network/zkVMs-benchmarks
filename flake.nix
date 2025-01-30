{
  description = "zkVMs benchmarks";

  nixConfig = {
    extra-substituters = "https://nix-blockchain-development.cachix.org";
    extra-trusted-public-keys = "nix-blockchain-development.cachix.org-1:Ekei3RuW3Se+P/UIo6Q/oAgor/fVhFuuuX5jR8K/cdg=";
  };

  inputs = {
    mcl-blockchain.url = "github:metacraft-labs/nix-blockchain-development?ref=zkvm-packages";
    nixpkgs.follows = "mcl-blockchain/nixpkgs";
    crane.follows = "mcl-blockchain/crane";
    rust-overlay.follows = "mcl-blockchain/rust-overlay";
    # flake-utils.follows = "mcl-blockchain/flake-utils";
  };

  outputs = { self, nixpkgs, mcl-blockchain, crane, rust-overlay, ... }:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { system = system; overlays = [
        mcl-blockchain.overlays.default
        rust-overlay.overlays.default
      ];
    };
    craneLib-default = crane.mkLib pkgs;
    callPackage = pkgs.lib.callPackageWith pkgs;

    zkvms = builtins.attrNames
      (pkgs.lib.filterAttrs
        (_: type: type == "directory")
        (builtins.readDir ./zkvms));

    guests = [ null ] ++ (builtins.attrNames
      (pkgs.lib.filterAttrs
        (_: type: type == "directory")
        (builtins.readDir ./guests)));

    foldr = pkgs.lib.foldr;

    createPackages = guestName: let
      guest = if guestName == null then "graph_coloring" else guestName;
      postfix = if guestName == null then "" else "/" + guest;

      args-zkVM = {
        inherit craneLib-default;
        zkvmLib = (import ./zkvmLib.nix) pkgs guest;
      };
    in foldr
      (host: accum: accum // {
          "${host}${postfix}" = callPackage ./zkvms/${host}/default.nix args-zkVM;
        })
      {}
      zkvms;

    hostPackages = foldr
      (guest: accum: accum // (createPackages guest))
      {}
      guests;

    guestPackages = foldr
      (guest: accum: accum // {
          ${guest} = callPackage ./guest.nix { inherit guest; inherit zkvms; inherit hostPackages; };
        })
      {}
      guests;
  in {
    packages.${system} = hostPackages // guestPackages;
  };
}
