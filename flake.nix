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

    fixDeps = commonArgs: commonArgs // {
        postUnpack = ''
          ln -s ../../../guests ./source/zkvms/${commonArgs.pname}/guest/
          ln -s ../../../guests_macro ./source/zkvms/${commonArgs.pname}/guest/
          ln -s ../../Cargo.lock ./source/zkvms/${commonArgs.pname}/
        '';

        preBuild = ''
          cd zkvms/${commonArgs.pname}
        '';
      };


    # Overrides build and install phases for use with zkVMs
    # Requirements:
    # - zkVM is inside zkvms/pname/
    # - guest crate is located at zkvms/pname/guest and is named "guest"
    withCustomPhases = currentPackage: with currentPackage; {
        buildGuestPhase = ''
          pushd guest
          runHook preBuildGuest

          cargo build --release --target ${currentPackage.guestTarget}
          ln -s ../../guest/target/${currentPackage.guestTarget}/release/guest ../host/src/guest

          runHook postBuildGuest
          popd
        '';

        buildPhase = ''
          export INPUTS="$PWD/Vertices-010.in"

          pushd zkvms/${currentPackage.pname}
          runPhase buildGuestPhase
          runHook preBuild

          cargo build --bin ${hostBin} --release

          runHook postBuild
          popd
        '';

        installPhase = ''
          runHook preInstall

          mkdir -p "$out"/bin
          for bin in $(find . -type f -regex "./zkvms/.*release/[^/]*" -executable -print)
          do
            mv "$bin" "$out"/bin/
          done

          cat <<EOF > "$out"/bin/${pname}
          #!/usr/bin/env sh
          ${if currentPackage ? preRun then preRun else ""}
          "$out"/bin/${hostBin} \$@
          EOF
          chmod +x "$out"/bin/${pname}

          runHook postInstall
        '';

        doNotPostBuildInstallCargoBinaries = true;
      } // currentPackage;


    args-zkVM = {
      craneLib-default = crane.mkLib pkgs;
      zkVM-helpers = {
        inherit fixDeps;
        inherit withCustomPhases;
      };
    };
  in {
    packages.${system} = {
      risc0 = callPackage ./zkvms/risc0/default.nix args-zkVM;
      sp1 = callPackage ./zkvms/sp1/default.nix args-zkVM;
    };
  };
}
