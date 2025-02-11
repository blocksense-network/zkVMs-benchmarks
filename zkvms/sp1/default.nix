{ zkvmLib,
  stdenv,
  lib,
  just,
  metacraft-labs,
  pkg-config,
  craneLib-default,
}:
let
  commonArgs = {
    pname = "sp1";
    inherit (metacraft-labs.sp1) version;

    src = with lib.fileset; toSource {
      root = ../..;
      fileset = intersection (gitTracked ../..) (unions [
          ./.
          ../../guests
          ../../guests_macro
          ../../zkvms_host_io
      ]);
    };
  };

  craneLib = craneLib-default.overrideToolchain metacraft-labs.sp1;
  cargoArtifacts = zkvmLib.buildDepsOnly craneLib commonArgs;
in
  zkvmLib.buildPackage craneLib (commonArgs
    // {
      inherit cargoArtifacts;

      nativeBuildInputs = [
        metacraft-labs.sp1
      ];

      guestTarget = "riscv32im-succinct-zkvm-elf";

      doCheck = false;
    })
