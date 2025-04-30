{
  description = "zkVMs benchmarks";

  nixConfig = {
    extra-substituters = "https://nix-blockchain-development.cachix.org";
    extra-trusted-public-keys =
      "nix-blockchain-development.cachix.org-1:Ekei3RuW3Se+P/UIo6Q/oAgor/fVhFuuuX5jR8K/cdg=";
  };

  inputs = {
    mcl-blockchain.url = "github:metacraft-labs/nix-blockchain-development";
    mcl-blockchain-old.url =
      "github:metacraft-labs/nix-blockchain-development?rev=f717747a4ce11d5764578d8ee1c505d00bf8a81e";
    nixpkgs.follows = "mcl-blockchain/nixpkgs";
    crane.follows = "mcl-blockchain/crane";
    fenix.follows = "mcl-blockchain/fenix";
    # flake-utils.follows = "mcl-blockchain/flake-utils";
  };

  outputs =
    { self, nixpkgs, mcl-blockchain, mcl-blockchain-old, crane, fenix, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        system = system;
        overlays = [
          mcl-blockchain.overlays.default
          fenix.overlays.default
          (_: _: {
            metacraft-labs-old =
              mcl-blockchain-old.legacyPackages.${system}.metacraft-labs;
          })
        ];
      };
      craneLib-default = crane.mkLib pkgs;
      callPackage = pkgs.lib.callPackageWith pkgs;

      zkvms = builtins.attrNames
        (pkgs.lib.filterAttrs (_: type: type == "directory")
          (builtins.readDir ./zkvms));

      guests = builtins.attrNames
        (pkgs.lib.filterAttrs (_: type: type == "directory")
          (builtins.readDir ./guests));

      foldr = pkgs.lib.foldr;

      createPackages = guestName:
        let
          guest = if guestName == null then "graph_coloring" else guestName;
          postfix = if guestName == null then "" else "/" + guest;

          args-zkVM = {
            inherit craneLib-default;
            zkvmLib = (import ./zkvmLib.nix) pkgs guest;
          };
        in foldr (host: accum:
          accum // {
            "${host}${postfix}" =
              callPackage ./zkvms/${host}/default.nix args-zkVM;
          }) { } zkvms;

      hostPackages =
        foldr (guest: accum: accum // (createPackages guest)) { } guests;

      guestPackages = foldr (guest: accum:
        accum // {
          ${guest} = callPackage ./zkvms_guest_io/default.nix {
            inherit guest;
            inherit zkvms;
            inherit hostPackages;
            inherit craneLib-default;
            rev = self.rev or self.dirtyRev;
          };
        }) { } guests;
    in {
      packages.${system} = hostPackages // guestPackages // {
        rust-format-all = callPackage ./rust-format-all.nix {
        };
        update-nix-dependencies = callPackage ./update-nix-dependencies.nix {
          zkvms = builtins.map (name: mcl-blockchain.packages.${system}.${name}) zkvms;
        };
      };

      formatter.${system} = pkgs.nixfmt;
    };
}
