{ zkvmLib, stdenv, lib, just, metacraft-labs, pkg-config, openssl
, craneLib-default, libcxx, }:
let
  commonArgs = {
    pname = "jolt";
    inherit (metacraft-labs.jolt) version;

    src = with lib.fileset;
      toSource {
        root = ../..;
        fileset = intersection (gitTracked ../..)
          (unions [ ./. ../../guests ../../guests_macro ../../zkvms_host_io ]);
      };

    nativeBuildInputs = [ metacraft-labs.jolt openssl pkg-config libcxx ];
  };

  craneLib = craneLib-default.overrideToolchain metacraft-labs.jolt;
  cargoArtifacts = zkvmLib.buildDepsOnly craneLib (commonArgs // {
    postConfigure = ''
      sed -i '/dependencies.guest/,+1d' zkvms/jolt/host/Cargo.toml
    '';
  });
in zkvmLib.buildPackage craneLib (commonArgs // {
  inherit cargoArtifacts;

  guestTarget = "riscv32im-jolt-zkvm-elf";
  guestExtraArgs = "--features guest";

  preBuildGuest = ''
    RUSTUP_TOOLCHAIN="x"
    RUSTFLAGS="-C link-arg=-T${
      ./guest/guest.ld
    } -C passes=lower-atomic -C panic=abort -C strip=symbols -C opt-level=z"
    export RUSTUP_TOOLCHAIN RUSTFLAGS
  '';

  preRunBinaries = [ metacraft-labs.jolt ];

  preRunLibraries = [ openssl ];

  preRun = ''
    export ELF_PATH="$out/bin/guest"
  '';

  doCheck = false;
})
