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

      postPatch = ''
        ln -s ../../../../guests/graph_coloring ./zkvms/nexus/guest/src/zkp
      '';

      hostBin = "host-nexus";
      guestTarget = "riscv32i-unknown-none-elf";
      extraGuestArgs = "-- --cfg 'feature=\"no_std\"' -C link-arg=-T${guest/guest.ld}";

      buildGuestPhase = ''
        pushd guest

        cargo rustc --release --target ${guestTarget} ${extraGuestArgs}
        ln -s ../../guest/target/${guestTarget}/release/guest ../host/src/guest

        popd
      '';

      preRun = ''
        export ELF_PATH="$out/bin/guest"
        export PKG_CONFIG_PATH='${openssl.dev}/lib/pkgconfig' # Dirty hack
        export LD_LIBRARY_PATH='${lib.makeLibraryPath [ openssl ]}'
      '';

      doCheck = false;
    }))
