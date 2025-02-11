{ zkvmLib,
  lib,
  just,
  metacraft-labs,
  rust-bin,
  protobuf,
  pkg-config,
  openssl,
  buildGoModule,
  fetchFromGitHub,
  craneLib-default,
}:
let
  zkm_libsnark = buildGoModule rec {
    pname = "zkm_libsnark";
    version = "0.1.0";

    src = fetchFromGitHub {
      owner = "zkMIPS";
      repo = "zkm-project-template";
      sparseCheckout = [ "sdk/src/local/libsnark" ];
      rev = "155221dfa05daf31d7bfe6b601116ef5a03b82c9";
      hash = "sha256-6hT7cMD3iXN65SFFgMHIPKzzQ212/uhZNIpjJNZ0Dek=";
    };

    sourceRoot = "${src.name}/sdk/src/local/libsnark";

    vendorHash = "sha256-tGajRfJ8G4M89QSiJnjpTzQ3+VA2RLkavD1ipANeOSI=";

    buildPhase = "sh ./compile.sh";

    installPhase = ''
      mkdir -p "$out"/lib
      mv libsnark.so "$out"/lib/
    '';
  };

  commonArgs = {
    pname = "zkm";
    inherit (metacraft-labs.zkm) version;

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
      pkg-config
      openssl
      protobuf
      metacraft-labs.zkm
    ];
  };

  craneLib = craneLib-default.overrideToolchain metacraft-labs.zkm;
  cargoArtifacts = zkvmLib.buildDepsOnly craneLib commonArgs;
in
  zkvmLib.buildPackage craneLib (commonArgs
    // {
      inherit cargoArtifacts;

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
    })
