{ zkvmLib, stdenv, lib, metacraft-labs, pkg-config, craneLib-default, }:
let
  commonArgs = {
    pname = "risc0";
    inherit (metacraft-labs.risc0) version;

    src = with lib.fileset;
      toSource {
        root = ../..;
        fileset = intersection (gitTracked ../..)
          (unions [ ./. ../../guests ../../guests_macro ../../zkvms_host_io ]);
      };
  };

  craneLib = craneLib-default.overrideToolchain metacraft-labs.risc0;
  cargoArtifacts = zkvmLib.buildDepsOnly craneLib commonArgs;
in zkvmLib.buildPackage craneLib (commonArgs // {
  inherit cargoArtifacts;

  nativeBuildInputs = [ metacraft-labs.risc0 ];

  guestToolchain = metacraft-labs.risc0-rust;

  guestTarget = "riscv32im-risc0-zkvm-elf";

  preBuildGuest = ''
    # Should be set only when RISC0 is compiled with unstable feature
    # https://github.com/risc0/risc0/blob/b5bf2d4a50cfb954da7f507766ba0f120c716958/risc0/build/src/lib.rs#L430-L435
    export RISC0_FEATURE_bigint2=""
  '';

  postBuildGuest = ''
    cd ../guest_elf_patch
    cargo run --release
  '';

  preBuild = ''
    # Used for verification
    # https://github.com/risc0/risc0/blob/881e512732eca72849b2d0e263a1242aba3158af/risc0/build/src/lib.rs#L192-L195
    export GUEST_ID="$(${metacraft-labs.risc0}/bin/r0vm --elf ./src/guest --id)"
  '';

  preRunBinaries = [ metacraft-labs.risc0 ];

  doCheck = false;
})
