{ zkvmLib, lib, just, metacraft-labs, metacraft-labs-old, protobuf, pkg-config
, openssl, buildGoModule, fetchFromGitHub, craneLib-default, }:
let
  commonArgs = {
    pname = "zkm";
    inherit (metacraft-labs.zkm) version;

    src = with lib.fileset;
      toSource {
        root = ../..;
        fileset = intersection (gitTracked ../..)
          (unions [ ./. ../../guests ../../guests_macro ../../zkvms_host_io ]);
      };

    nativeBuildInputs = [ pkg-config openssl protobuf metacraft-labs.zkm ];
  };

  craneLib = craneLib-default.overrideToolchain metacraft-labs.zkm;
  cargoArtifacts = zkvmLib.buildDepsOnly craneLib commonArgs;
in zkvmLib.buildPackage craneLib (commonArgs // {
  inherit cargoArtifacts;

  guestTarget = "mips-zkm-zkvm-elf";

  preBuildGuest = ''
    # https://github.com/zkMIPS/zkm/blob/0e62a053970eb25c81aa409d0c7234f5611a192d/build/src/command/utils.rs#L45-L61
    export RUSTFLAGS="-C target-cpu=mips2 -C target-feature=+crt-static -C link-arg=-nostdlib -C link-arg=-g -C link-arg=--entry=main"
  '';

  preBuild = ''
    export RUSTFLAGS="-L ${metacraft-labs.zkm}/lib"
  '';

  hostToolchain = metacraft-labs-old.zkm;

  preRunLibraries = [ openssl metacraft-labs.zkm ];

  preRun = ''
    export ELF_PATH="$out/bin/guest"
    export PKG_CONFIG_PATH='${openssl.dev}/lib/pkgconfig' # Dirty hack

    echo "Generating witness. THIS COULD RETURN A SIGSEGV ERROR, IGNORE IT"
    SNARK_SETUP=true "$out"/bin/host-${commonArgs.pname} prove 2>/dev/null || true
  '';

  doCheck = false;
})
