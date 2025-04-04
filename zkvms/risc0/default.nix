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
