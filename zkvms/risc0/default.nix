{ stdenv,
  lib,
  just,
  metacraft-labs,
  pkg-config,
  craneLib-default,
  withZKVMPhases,
}:
let
  fs = lib.fileset;

  commonArgs = rec {
    pname = "risc0";
    version = "infdev";

    src = fs.toSource {
      root = ../..;
      fileset = fs.intersection (fs.gitTracked ../..) (fs.unions [
          ./.
          ../../guests
          ../../guests_macro
          ../../Cargo.lock
          ../../Cargo.toml
          ../../Vertices-010.in
      ]);
    };
  };

  craneLib = craneLib-default.overrideToolchain metacraft-labs.risc0;
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
  craneLib.buildPackage (withZKVMPhases (commonArgs
    // {
      inherit cargoArtifacts;

      nativeBuildInputs = [
        metacraft-labs.risc0
      ];

      postPatch = ''
        ln -s ../../../../guests/graph_coloring ./zkvms/risc0/guest/src/zkp
      '';

      preBuild = ''
        cd zkvms/risc0/guest
        cargo build --release --target riscv32im-risc0-zkvm-elf
        ln -s ../../../../zkvms/risc0/guest/target/riscv32im-risc0-zkvm-elf/release/guest ../host/src/guest
      '';

      hostBin = "host-risc0";

      preRun = ''
        export PATH="\$PATH:${metacraft-labs.risc0}/bin"
      '';

      doCheck = false;
    }))
