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
    callPackage = pkgs.lib.callPackageWith pkgs;

    fixDeps = commonArgs: commonArgs // {
        postUnpack = ''
          ${commonArgs.postUnpack or ""}
          ln -s ../../../guests ./source/zkvms/${commonArgs.pname}/guest/
          ln -s ../../../guests_macro ./source/zkvms/${commonArgs.pname}/guest/
          ln -s ../../Cargo.lock ./source/zkvms/${commonArgs.pname}/
        '';

        preBuild = ''
          ${commonArgs.preBuild or ""}
          cd zkvms/${commonArgs.pname}
        '';
      };


    # Creates custom build and install phases
    # Adds the "buildGuest" phase
    # Adds the "run" pseudo-phase (running your zkVM is done with a shell script,
    #     this "phase" allows one to add things to the script)
    # Requirements:
    # - zkVM is inside zkvms/pname/
    # - guest crate is located at zkvms/pname/guest and is named "guest"
    withCustomPhases = guest: currentPackage: let
        hostBin = currentPackage.hostBin or ("host-" + currentPackage.pname);
        zkpPath = "zkvms/${currentPackage.pname}/guest/src/zkp";
      in with currentPackage; {
        phases = [
          "unpackPhase" # Standard phases
          "linkGuest" # Custom phase
          "patchPhase" "configurePhase" # Standard phases
          "buildGuestPhase" # Custom phase
          "buildPhase" "checkPhase" "installPhase" "fixupPhase" # Standard phases
        ];

        linkGuest = ''
          ln -s ../../../../guests/${guest} ./${zkpPath}
        '';

        buildGuestPhase = ''
          pushd zkvms/${currentPackage.pname}/guest
          runHook preBuildGuest

          ${currentPackage.buildGuestCommand or "cargo build --release"} \
              ${if currentPackage ? guestTarget then "--target " + currentPackage.guestTarget else ""} \
              ${currentPackage.guestExtraArgs or ""}

          ${if currentPackage ? guestTarget then "ln -s ../../guest/target/${currentPackage.guestTarget}/release/guest ../host/src/guest" else ""}
          unset RUSTUP_TOOLCHAIN RUSTFLAGS CARGO_ENCODED_RUSTFLAGS

          runHook postBuildGuest
          popd
        '';

        buildPhase = ''
          export INPUTS="$PWD/Vertices-010.in"

          pushd zkvms/${currentPackage.pname}
          runHook preBuild

          cargo build --bin ${hostBin} --release

          runHook postBuild
          popd
        '';

        installPhase = let
          preRunBinaries =
            if currentPackage ? preRunBinaries && builtins.length currentPackage.preRunBinaries > 0 then
              "export PATH=\"\\$PATH:" + pkgs.lib.makeBinPath currentPackage.preRunBinaries + "\""
            else
              "";
          preRunLibraries =
            if currentPackage ? preRunLibraries && builtins.length currentPackage.preRunLibraries > 0 then
              "export LD_LIBRARY_PATH=\"\\$LD_LIBRARY_PATH:" + pkgs.lib.makeLibraryPath currentPackage.preRunLibraries + "\""
            else
              "";
        in ''
          runHook preInstall

          mkdir -p "$out"/bin
          for bin in $(find . -type f -regex "./zkvms/.*release/[^/]*" -executable -print)
          do
            mv "$bin" "$out"/bin/
          done

          cat <<EOF > "$out"/bin/${pname}
          #!/usr/bin/env sh
          ${preRunBinaries}
          ${preRunLibraries}
          ${currentPackage.preRun or ""}
          "$out"/bin/${hostBin} \$@
          EOF
          chmod +x "$out"/bin/${pname}

          runHook postInstall
        '';

        doNotPostBuildInstallCargoBinaries = true;
      } // currentPackage;

    createPackages = guestName: let
      guest = if guestName == null then "graph_coloring" else guestName;
      postfix = if guestName == null then "" else "/" + guest;

      args-zkVM = {
        craneLib-default = crane.mkLib pkgs;
        zkVM-helpers = {
          inherit fixDeps;
          withCustomPhases = withCustomPhases guest;
        };
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
