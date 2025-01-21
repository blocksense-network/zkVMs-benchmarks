{ zkVM-helpers,
  stdenv,
  lib,
  just,
  metacraft-labs,
  rust-bin,
  protobuf,
  pkg-config,
  openssl,
  buildGoModule,
  craneLib-default,
}:
let
  zkm_libsnark = buildGoModule {
    pname = "zkm_libsnark";
    version = "0.1.0";
    src = with lib.fileset; toSource {
      root = ./sdk/src/local/libsnark;
      fileset = ./sdk/src/local/libsnark;
    };
    vendorHash = "sha256-tGajRfJ8G4M89QSiJnjpTzQ3+VA2RLkavD1ipANeOSI=";

    buildPhase = "sh ./compile.sh";
    installPhase = ''
      mkdir -p "$out"/lib
      mv libsnark.so "$out"/lib/
    '';
  };

  commonArgs = {
    pname = "zkm";
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
      pkg-config
      openssl
      protobuf
      metacraft-labs.zkm
    ];
  };

  craneLib = craneLib-default.overrideToolchain metacraft-labs.zkm;
  cargoArtifacts = craneLib.buildDepsOnly (zkVM-helpers.fixDeps commonArgs);
in
  craneLib.buildPackage (zkVM-helpers.withCustomPhases (commonArgs
    // {
      inherit cargoArtifacts;

      postPatch = ''
        ln -s ../../../../guests/graph_coloring ./zkvms/zkm/guest/src/zkp
      '';

      guestTarget = "mips-zkm-zkvm-elf";

      preBuildGuest = ''
        # https://github.com/zkMIPS/zkm/blob/0e62a053970eb25c81aa409d0c7234f5611a192d/build/src/command/utils.rs#L45-L61
        export RUSTFLAGS="-C target-cpu=mips2 -C target-feature=+crt-static -C link-arg=-nostdlib -C link-arg=-g -C link-arg=--entry=main"
      '';

      preBuild = ''
        export RUSTFLAGS="-L ${zkm_libsnark}/lib"
      '';

      preRunLibraries = [
        openssl
        zkm_libsnark
      ];

      preRun = ''
        export ELF_PATH="$out/bin/guest"
        export PKG_CONFIG_PATH='${openssl.dev}/lib/pkgconfig' # Dirty hack
      '';

      doCheck = false;
    }))
