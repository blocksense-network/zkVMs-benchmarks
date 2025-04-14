pkgs: guest:
let
  generateCargoLocks = craneLib: args: rec {
    cargoLockDrv = pkgs.stdenv.mkDerivation {
      name = "CargoLocks-${args.pname}";
      src = pkgs.lib.fileset.toSource {
        root = ./.;
        fileset = ./.;
      };

      installPhase = let
        # Since we're concatenating Cargo.lock files, duplicate package entries
        # are inevitable and cargo crashes when it encounters them.
        # We'll manually remove all duplicates and cargo will be happy.
        # This is a disgusting hack, but it's the best I've come up with.
        removeDuplicates = ''
          BEGIN {
              unique = 1
          }

          /^\[\[package\]\]/ { unique = 0; next }

          /^name = / {
              match($0, /".*"/)
              name = substr($0, RSTART + 1, RLENGTH - 2)
              next
          }

          name && /^version = / {
              match($0, /".*"/)
              version = substr($0, RSTART + 1, RLENGTH - 2)
              next
          }

          version && /^source = / {
              match($0, /".*"/)
              source = substr($0, RSTART + 1, RLENGTH - 2)
              next
          }

          source && /^checksum = / {
              match($0, /".*"/)
              checksum = substr($0, RSTART + 1, RLENGTH - 2)
              next
          }

          name && !unique {
              unique = (index(versions[name], version) == 0) ||
                       (source && index(sources[name], source) == 0) ||
                       (checksum && index(checksums[name], checksum) == 0)

              if (unique) {
                  versions[name]  = versions[name] version
                  sources[name]   = sources[name] source
                  checksums[name] = checksums[name] checksum

                  print "[[package]]"
                  print "name = \"" name "\""
                  print "version = \"" version "\""
                  if (source)   print "source = \"" source "\""
                  if (checksum) print "checksum = \"" checksum "\""
              }
              name = ""; version = ""; source = ""; checksum = ""
          }

          unique || /^$/ { print }
        '';
      in ''
        mkdir -p "$out"
        cd zkvms/${args.pname}

      '' + (if args ? extraLockfile then ''
        cat ${args.extraLockfile} > lockfile
        tail -n +4 ./host/Cargo.lock >> lockfile
      '' else ''
        cat ./host/Cargo.lock > lockfile
      '') + ''

        tail -n +4 ./guest/Cargo.lock >> lockfile
        tail -n +4 ../../guests/${guest}/Cargo.lock >> lockfile

        awk '${removeDuplicates}' lockfile > "$out/Cargo.lock"
      '';
    };

    cargoVendorDir = craneLib.vendorCargoDeps ({
      src = cargoLockDrv;
    } // (if args ? overrideVendorCargoPackage then {
      inherit (args) overrideVendorCargoPackage;
    } else
      { }) // (if args ? overrideVendorGitCheckout then {
        inherit (args) overrideVendorGitCheckout;
      } else
        { }));

  };
in {
  buildDepsOnly = craneLib: args:
    if builtins.pathExists ./guests/${guest}/.no_${args.pname} then
      { }
    else
      let cargoLocks = generateCargoLocks craneLib args;
      in craneLib.buildDepsOnly (cargoLocks // (builtins.removeAttrs args [
        "overrideVendorCargoPackage"
        "overrideVendorGitCheckout"
      ]) // {
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

  buildPackage = craneLib: args:
    if builtins.pathExists ./guests/${guest}/.no_${args.pname} then
      pkgs.writeShellApplication {
        name = "${args.pname}_${guest}";

        text =
          let contents = builtins.readFile ./guests/${guest}/.no_${args.pname};
          in ''
            echo 'Guest program "${guest}" has blacklisted ${args.pname} zkVM!'

          '' + (if builtins.stringLength contents > 1 then ''
            echo
            echo 'Reason:'
            echo '${contents}'
          '' else
            "");
      }
    else
      let
        pname = "${args.pname}_${guest}";
        cargoLocks = generateCargoLocks craneLib args;
      in craneLib.buildPackage (cargoLocks // {
        phases = [
          "unpackPhase"
          "patchPhase"
          "configurePhase" # Standard phases
          "cargoSetupGuest"
          "buildGuestPhase" # Custom phases
          "buildPhase"
          "checkPhase"
          "installPhase"
          "fixupPhase" # Standard phases
        ];

        cargoSetupGuest = let
          appended = ''
            [features]
            guest = [] # Only used in jolt
            no_std = ["zkp/no_std"]
            ${args.pname} = ["zkp/${args.pname}"]
          '';
        in ''
          pushd zkvms/${args.pname}/guest

          cp '${cargoLocks.cargoLockDrv}/Cargo.lock' Cargo.lock
          cargo add --path "../../../guests/${guest}" --rename zkp --offline --features '${args.pname}'
          echo '${appended}' >> Cargo.toml

          popd
        '';

        buildGuestPhase = ''
          export INPUTS_DIR="$PWD/guests/${guest}"
          export ZKVM="${args.pname}" GUEST="${guest}"
          OLD_PATH="$PATH"

          ${if args ? guestToolchain then
            ''export PATH="${args.guestToolchain}/bin:$PATH"''
          else
            ""}

          pushd zkvms/${args.pname}/guest
          runHook preBuildGuest

          ${
            args.buildGuestCommand or "cargo build --release --features ${args.pname}"
          } \
              ${
                if args ? guestTarget then
                  "--target " + args.guestTarget
                else
                  ""
              } \
              ${args.guestExtraArgs or ""}

          ${if args ? guestTarget then
            "ln -s ../../guest/target/${args.guestTarget}/release/guest ../host/src/guest"
          else
            ""}
          unset RUSTUP_TOOLCHAIN RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
          export PATH="$OLD_PATH"

          runHook postBuildGuest
          popd
        '';

        buildPhase = ''
          export INPUTS_DIR="$PWD/guests/${guest}"
          export ZKVM="${args.pname}" GUEST="${guest}"
          OLD_PATH="$PATH"

          ${if args ? hostToolchain then
            ''export PATH="${args.hostToolchain}/bin:$PATH"''
          else
            ""}

          pushd zkvms/${args.pname}/host
          runHook preBuild

          cargo --version
          cargo build --release

          export PATH="$OLD_PATH"

          runHook postBuild
          popd
        '';

        installPhase = let
          preRunBinaries = if args ? preRunBinaries
          && builtins.length args.preRunBinaries > 0 then
            ''export PATH="\$PATH:'' + pkgs.lib.makeBinPath args.preRunBinaries
            + ''"''
          else
            "";
          preRunLibraries = if args ? preRunLibraries
          && builtins.length args.preRunLibraries > 0 then
            ''export LD_LIBRARY_PATH="\$LD_LIBRARY_PATH:''
            + pkgs.lib.makeLibraryPath args.preRunLibraries + ''"''
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
      } // (builtins.removeAttrs args [
        "overrideVendorCargoPackage"
        "overrideVendorGitCheckout"
      ]) // {
        inherit pname;
      });
}
