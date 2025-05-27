{
  description = "zkVMs benchmarks";

  nixConfig = {
    extra-substituters = "https://nix-blockchain-development.cachix.org";
    extra-trusted-public-keys = "nix-blockchain-development.cachix.org-1:Ekei3RuW3Se+P/UIo6Q/oAgor/fVhFuuuX5jR8K/cdg=";
  };

  inputs = {
    mcl-blockchain.url = "github:metacraft-labs/nix-blockchain-development";
    mcl-blockchain-old.url = "github:metacraft-labs/nix-blockchain-development?rev=f717747a4ce11d5764578d8ee1c505d00bf8a81e";
    nixpkgs.follows = "mcl-blockchain/nixpkgs";
    crane.follows = "mcl-blockchain/crane";
    fenix.follows = "mcl-blockchain/fenix";
    flake-parts.follows = "mcl-blockchain/flake-parts";
    systems.url = "github:nix-systems/default";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;

      perSystem =
        {
          inputs',
          pkgs,
          lib,
          system,
          ...
        }:
        let
          craneLib-default = inputs.crane.mkLib pkgs;
          callPackage = lib.callPackageWith pkgs;

          zkvms = builtins.attrNames (
            lib.filterAttrs (_: type: type == "directory") (builtins.readDir ./zkvms)
          );

          guests = builtins.attrNames (
            lib.filterAttrs (_: type: type == "directory") (builtins.readDir ./guests)
          );

          createPackages =
            guestName:
            let
              guest = if guestName == null then "graph_coloring" else guestName;
              postfix = if guestName == null then "" else "/" + guest;

              args-zkVM = {
                inherit craneLib-default;
                zkvmLib = (import ./zkvmLib.nix) pkgs guest;
              };
            in
            lib.foldr (
              host: accum:
              accum
              // {
                "${host}${postfix}" = callPackage ./zkvms/${host}/default.nix args-zkVM;
              }
            ) { } zkvms;

          hostPackages = lib.foldr (guest: accum: accum // (createPackages guest)) { } guests;

          guestPackages = lib.foldr (
            guest: accum:
            accum
            // {
              ${guest} = callPackage ./zkvms_guest_io/default.nix {
                inherit guest;
                inherit zkvms;
                inherit hostPackages;
                inherit craneLib-default;
                rev = inputs.self.rev or inputs.self.dirtyRev;
              };
            }
          ) { } guests;
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              inputs.mcl-blockchain.overlays.default
              inputs.fenix.overlays.default
              (_: _: {
                metacraft-labs-old = inputs'.mcl-blockchain-old.legacyPackages.metacraft-labs;
              })
            ];
          };

          packages =
            hostPackages
            // guestPackages
            // {
              rust-format-all = callPackage ./rust-format-all.nix { };
              update-nix-dependencies = callPackage ./update-nix-dependencies.nix {
                zkvms = builtins.map (name: inputs.mcl-blockchain.packages.${system}.${name}) zkvms;
              };
            };

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              (fenix.stable.withComponents [
                "cargo"
                "clippy"
                "rust-src"
                "rustc"
                "rustfmt"
                "rust-analyzer"
              ])

              # Possibly required system libraries by Rust crypto/networking crates
              pkg-config
              openssl
            ];

            shellHook = ''
              echo "zkVMs benchmarks development environment"
              echo "Available packages: ${builtins.concatStringsSep ", " (builtins.attrNames hostPackages)}"
            '';
          };

          formatter = pkgs.nixfmt-rfc-style;
        };
    };
}
