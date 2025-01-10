{ stdenv,
  lib,
  just,
  metacraft-labs,
  pkg-config,
  craneLib-default,
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
  craneLib.buildPackage (commonArgs
    // {
      inherit cargoArtifacts;

      nativeBuildInputs = [
        just
        metacraft-labs.risc0
        stdenv.cc
        pkg-config
      ];

      postPatch = ''
        ln -s ../../../../guests/graph_coloring ./zkvms/risc0/guest/src/zkp
      '';

      preBuild = ''
        INPUTS="$PWD/Vertices-010.in"
        export INPUTS
        cd zkvms/risc0
        just prove
      '';

      doCheck = false;
    })
