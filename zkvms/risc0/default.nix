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
    pname = "risc0";
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
  };

  craneLib = craneLib-default.overrideToolchain metacraft-labs.risc0;
  cargoArtifacts = craneLib.buildDepsOnly (zkVM-helpers.fixDeps commonArgs);
in
  craneLib.buildPackage (zkVM-helpers.withCustomPhases (commonArgs
    // {
      inherit cargoArtifacts;

      nativeBuildInputs = [
        metacraft-labs.risc0
      ];

      postPatch = ''
        ln -s ../../../../guests/graph_coloring ./zkvms/risc0/guest/src/zkp
      '';

      preBuild = ''
        pushd ./guest
        cargo build --release --target riscv32im-risc0-zkvm-elf
        ln -s ../../../../zkvms/risc0/guest/target/riscv32im-risc0-zkvm-elf/release/guest ../host/src/guest
        popd

        # Used for verification
        # https://github.com/risc0/risc0/blob/881e512732eca72849b2d0e263a1242aba3158af/risc0/build/src/lib.rs#L192-L195
        export GUEST_ID="$(${metacraft-labs.risc0}/bin/r0vm --elf ../host/src/guest --id)"
      '';

      hostBin = "host-risc0";

      preRun = ''
        export PATH="\$PATH:${metacraft-labs.risc0}/bin"
      '';

      doCheck = false;
    }))
