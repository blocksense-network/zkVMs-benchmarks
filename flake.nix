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

    createPackages = guestName: let
      guest = if guestName == null then "graph_coloring" else guestName;
      postfix = if guestName == null then "" else "/" + guest;

      args-zkVM = {
        inherit craneLib-default;
        zkvmLib = (import ./zkvmLib.nix) pkgs guest;
      };
    in {
      "risc0${postfix}" = callPackage ./zkvms/risc0/default.nix args-zkVM;
      "sp1${postfix}" = callPackage ./zkvms/sp1/default.nix args-zkVM;
      "zkwasm${postfix}" = callPackage ./zkvms/zkwasm/default.nix args-zkVM;
      "zkm${postfix}" = callPackage ./zkvms/zkm/default.nix args-zkVM;
      "jolt${postfix}" = callPackage ./zkvms/jolt/default.nix args-zkVM;
      "nexus${postfix}" = callPackage ./zkvms/nexus/default.nix args-zkVM;
    };

    guests = [ null ] ++ (builtins.attrNames
      (pkgs.lib.filterAttrs
        (_: type: type == "directory")
        (builtins.readDir ./guests)));
  in {
    packages.${system} = pkgs.lib.foldr
      (guest: accum: accum // (createPackages guest))
      {}
      guests;
  };
}
