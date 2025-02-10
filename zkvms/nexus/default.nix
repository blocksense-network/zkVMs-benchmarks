{ zkvmLib,
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
      ]);
    };

    nativeBuildInputs = [
      metacraft-labs.nexus
      openssl
      pkg-config
    ];
  };

  craneLib = craneLib-default.overrideToolchain metacraft-labs.nexus;
  cargoArtifacts = zkvmLib.buildDepsOnly craneLib commonArgs;
in
  zkvmLib.buildPackage craneLib (commonArgs
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
    })
