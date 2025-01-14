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
          ../../Vertices-010.in
      ]);
    };

    cargoLock = ./Cargo.lock;

    preBuild = ''
      cd zkvms/sp1
    '';
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

      preBuild = ''
        pushd ./guest
        cargo build --release --target riscv32im-succinct-zkvm-elf
        ln -s ../../../../zkvms/sp1/guest/target/riscv32im-succinct-zkvm-elf/release/guest ../host/src/guest
        popd
      '';

      hostBin = "host-sp1";

      doCheck = false;
    }))
