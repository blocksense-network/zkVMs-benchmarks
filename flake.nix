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

  outputs = { self, nixpkgs, mcl-blockchain, crane, ... }:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { system = system; overlays = [ mcl-blockchain.overlays.default ]; };
    callPackage = pkgs.lib.callPackageWith pkgs;

    withZKVMPhases = currentPackage: with currentPackage; {
        buildPhase = ''
          root_dir="$PWD"
          runHook preBuild
          cd "$root_dir"

          export INPUTS="$PWD/Vertices-010.in"
          cargo build --bin ${hostBin} --release

          runHook postBuild
        '';

        installPhase = ''
          runHook preInstall
          mkdir -p "$out"/bin
          for bin in $(find . -type f -regex ".*release/[^/]*" -executable -print)
          do
            mv "$bin" "$out"/bin/
          done
          runHook postInstall
        '';

        doNotPostBuildInstallCargoBinaries = true;
      } // currentPackage;

    args-zkVM = {
      craneLib-default = crane.mkLib pkgs;
      inherit withZKVMPhases;
    };
  in {
    packages.${system}.risc0 = callPackage ./zkvms/risc0/default.nix args-zkVM;
  };
}
