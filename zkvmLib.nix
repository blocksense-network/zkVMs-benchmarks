pkgs: guest: let
  generateCargoLocks = craneLib: args: rec {
    cargoLockDrv = pkgs.stdenv.mkDerivation {
        name = "CargoLocks-${args.pname}";
        src = pkgs.lib.fileset.toSource {
          root = ./.;
          fileset = ./.;
        };

        installPhase = ''
          mkdir -p "$out"
          cd zkvms/${args.pname}

          cat ./host/Cargo.lock > "$out/Cargo.lock"
          tail -n +4 ./guest/Cargo.lock >> "$out/Cargo.lock"
          tail -n +4 ../../guests/${guest}/Cargo.lock >> "$out/Cargo.lock"

        '';
      };

    cargoVendorDir = craneLib.vendorCargoDeps {
      src = cargoLockDrv;
    };
  };
in {
  buildDepsOnly = craneLib: args: let
    cargoLocks = generateCargoLocks craneLib args;
  in craneLib.buildDepsOnly (cargoLocks // args // {
      postUnpack = ''
        ${args.postUnpack or ""}
        ln -s ../../../guests ./source/zkvms/${args.pname}/guest/
        ln -s ../../../guests_macro ./source/zkvms/${args.pname}/guest/

        cp '${cargoLocks.cargoLockDrv}/Cargo.lock' ./source/zkvms/${args.pname}/guest/Cargo.lock
        chmod +w ./source/zkvms/${args.pname}/guest/Cargo.lock
      '';

      preBuild = ''
        ${args.preBuild or ""}
        cd zkvms/${args.pname}/guest
        cargo check --release --offline --all-targets
      '';
  } // {
    pname = "${args.pname}_${guest}";
  });

  buildPackage = craneLib: args: let
    pname = "${args.pname}_${guest}";
  in craneLib.buildPackage ((generateCargoLocks craneLib args) // {
    phases = [
      "unpackPhase" # Standard phases
      "linkGuest" # Custom phase
      "patchPhase" "configurePhase" # Standard phases
      "buildGuestPhase" # Custom phase
      "buildPhase" "checkPhase" "installPhase" "fixupPhase" # Standard phases
    ];

    linkGuest = let
      appended = ''
        zkp = { path = "../../../guests/${guest}", package = "${guest}" }

        [features]
        guest = [] # Only used in jolt
        no_std = ["zkp/no_std"]
      '';
    in ''
      echo '${appended}' >> zkvms/${args.pname}/guest/Cargo.toml
    '';

    buildGuestPhase = ''
      export INPUTS_DIR="$PWD/guests/${guest}"
      export ZKVM="${args.pname}" GUEST="${guest}"

      pushd zkvms/${args.pname}/guest
      runHook preBuildGuest

      ${args.buildGuestCommand or "cargo build --release"} \
          ${if args ? guestTarget then "--target " + args.guestTarget else ""} \
          ${args.guestExtraArgs or ""}

      ${if args ? guestTarget then "ln -s ../../guest/target/${args.guestTarget}/release/guest ../host/src/guest" else ""}
      unset RUSTUP_TOOLCHAIN RUSTFLAGS CARGO_ENCODED_RUSTFLAGS

      runHook postBuildGuest
      popd
    '';

    buildPhase = ''
      export INPUTS_DIR="$PWD/guests/${guest}"
      export ZKVM="${args.pname}" GUEST="${guest}"

      pushd zkvms/${args.pname}/host
      runHook preBuild

      cargo build --release

      runHook postBuild
      popd
    '';

    installPhase = let
      preRunBinaries =
        if args ? preRunBinaries && builtins.length args.preRunBinaries > 0 then
          "export PATH=\"\\$PATH:" + pkgs.lib.makeBinPath args.preRunBinaries + "\""
        else
          "";
      preRunLibraries =
        if args ? preRunLibraries && builtins.length args.preRunLibraries > 0 then
          "export LD_LIBRARY_PATH=\"\\$LD_LIBRARY_PATH:" + pkgs.lib.makeLibraryPath args.preRunLibraries + "\""
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
      ${args.preRun or ""}
      "$out"/bin/host-${args.pname} \$@
      EOF
      chmod +x "$out"/bin/${pname}

      runHook postInstall
    '';

    doNotPostBuildInstallCargoBinaries = true;
  } // args // { inherit pname; });
}
