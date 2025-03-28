{ zkvmLib, stdenv, lib, just, metacraft-labs, pkg-config, craneLib-default
, gnum4, }:
let
  commonArgs = {
    pname = "sp1";
    inherit (metacraft-labs.sp1) version;

    nativeBuildInputs = [ metacraft-labs.sp1 gnum4 ];

    src = with lib.fileset;
      toSource {
        root = ../..;
        fileset = intersection (gitTracked ../..)
          (unions [ ./. ../../guests ../../guests_macro ../../zkvms_host_io ]);
      };

    extraLockfile = "${metacraft-labs.sp1}/Cargo.lock";
  };

  craneLib = craneLib-default.overrideToolchain metacraft-labs.sp1;
  cargoArtifacts = zkvmLib.buildDepsOnly craneLib commonArgs;
in zkvmLib.buildPackage craneLib (commonArgs // {
  inherit cargoArtifacts;

  guestTarget = "riscv32im-succinct-zkvm-elf";

  doCheck = false;
})
