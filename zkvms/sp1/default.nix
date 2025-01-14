{ zkVM-helpers,
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
    version = "infdev";

    src = with lib.fileset; toSource {
      root = ../..;
      fileset = intersection (gitTracked ../..) (unions [
          ./.
          ../../guests
          ../../guests_macro
          ../../zkvms_host_io
          ../../Vertices-010.in
      ]);
    };

    cargoLock = ./Cargo.lock;
  };

  craneLib = craneLib-default.overrideToolchain metacraft-labs.sp1;
  cargoArtifacts = craneLib.buildDepsOnly (zkVM-helpers.fixDeps commonArgs);
in
  craneLib.buildPackage (zkVM-helpers.withCustomPhases (commonArgs
    // {
      inherit cargoArtifacts;

      nativeBuildInputs = [
        metacraft-labs.sp1
      ];

      postPatch = ''
        ln -s ../../../../guests/graph_coloring ./zkvms/sp1/guest/src/zkp
      '';

      hostBin = "host-sp1";
      guestTarget = "riscv32im-succinct-zkvm-elf";

      doCheck = false;
    }))
