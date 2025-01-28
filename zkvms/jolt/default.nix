{ zkVM-helpers,
  stdenv,
  lib,
  just,
  metacraft-labs,
  pkg-config,
  openssl,
  craneLib-default,
}:
let
  commonArgs = {
    pname = "jolt";
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
      metacraft-labs.jolt
      openssl
      pkg-config
    ];
  };

  craneLib = craneLib-default.overrideToolchain metacraft-labs.jolt;
  cargoArtifacts = craneLib.buildDepsOnly (zkVM-helpers.fixDeps (commonArgs // {
    postConfigure = ''
      sed -i 's/"guest",//' zkvms/jolt/Cargo.toml
      sed -i '/dependencies.guest/,+1d' zkvms/jolt/host/Cargo.toml
    '';
  }));
in
  craneLib.buildPackage (zkVM-helpers.withCustomPhases (commonArgs
    // {
      inherit cargoArtifacts;

      postPatch = ''
        sed -i '/guest\/guests/d' ./zkvms/jolt/Cargo.toml
      '';

      guestTarget = "riscv32im-jolt-zkvm-elf";
      guestExtraArgs = "--features guest";

      preBuildGuest = ''
        RUSTUP_TOOLCHAIN="x"
        RUSTFLAGS="-C link-arg=-T${./guest/guest.ld} -C passes=lower-atomic -C panic=abort -C strip=symbols -C opt-level=z"
        export RUSTUP_TOOLCHAIN RUSTFLAGS
      '';

      preRunBinaries = [
        metacraft-labs.jolt
      ];

      preRun = ''
        export ELF_PATH="$out/bin/guest"
      '';

      doCheck = false;
    }))
