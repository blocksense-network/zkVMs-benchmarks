{ zkVM-helpers,
  stdenv,
  lib,
  just,
  metacraft-labs,
  openssl,
  pkg-config,
  craneLib-default,
}:
let
  commonArgs = {
    pname = "nexus";
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

    nativeBuildInputs = [
      metacraft-labs.nexus
      openssl
      pkg-config
    ];
  };

  craneLib = craneLib-default.overrideToolchain metacraft-labs.nexus;
  cargoArtifacts = craneLib.buildDepsOnly (zkVM-helpers.fixDeps commonArgs);
in
  craneLib.buildPackage (zkVM-helpers.withCustomPhases (commonArgs
    // rec {
      inherit cargoArtifacts;

      preBuildGuest = ''
        export RUSTFLAGS="-C link-arg=-T${guest/guest.ld}"
      '';

      guestTarget = "riscv32i-unknown-none-elf";
      guestExtraArgs = "--features no_std";

      preRunLibraries = [
        openssl
      ];

      preRun = ''
        export ELF_PATH="$out/bin/guest"
        export PKG_CONFIG_PATH='${openssl.dev}/lib/pkgconfig' # Dirty hack
      '';

      doCheck = false;
    }))
