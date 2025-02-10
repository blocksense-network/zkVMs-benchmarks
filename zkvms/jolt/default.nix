{ zkvmLib,
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
      ]);
    };

    nativeBuildInputs = [
      metacraft-labs.jolt
      openssl
      pkg-config
    ];
  };

  craneLib = craneLib-default.overrideToolchain metacraft-labs.jolt;
  cargoArtifacts = zkvmLib.buildDepsOnly craneLib (commonArgs // {
    postConfigure = ''
      sed -i '/dependencies.guest/,+1d' zkvms/jolt/host/Cargo.toml
    '';
  });
in
  zkvmLib.buildPackage craneLib (commonArgs
    // {
      inherit cargoArtifacts;

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
    })
