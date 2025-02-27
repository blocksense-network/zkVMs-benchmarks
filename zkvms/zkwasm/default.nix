{ zkvmLib, lib, rust-bin, metacraft-labs, wasm-pack, wasm-bindgen-cli, binaryen
, craneLib-default, stdenv, }:
let
  commonArgs = rec {
    pname = "zkwasm";
    inherit (metacraft-labs.zkwasm) version;

    src = with lib.fileset;
      toSource {
        root = ../..;
        fileset = intersection (gitTracked ../..)
          (unions [ ./. ../../guests ../../guests_macro ../../zkvms_host_io ]);
      };
  };

  rust-toolchain = rust-bin.nightly."2024-04-09".default.override {
    targets = [ "wasm32-unknown-unknown" ];
  };
  craneLib = craneLib-default.overrideToolchain rust-toolchain;
  cargoArtifacts = zkvmLib.buildDepsOnly craneLib commonArgs;
in zkvmLib.buildPackage craneLib (commonArgs // {
  inherit cargoArtifacts;

  nativeBuildInputs =
    [ metacraft-labs.zkwasm wasm-pack wasm-bindgen-cli binaryen ];

  preBuildGuest = ''
    # Workaround from
    # https://github.com/rustwasm/wasm-pack/issues/1335
    export WASM_PACK_CACHE=.wasm-pack-cache
  '';

  buildGuestCommand = "wasm-pack build --release --frozen --features zkwasm";

  preBuild = ''
    export GUEST_PATH="$out/pkg/guest_bg.wasm"
  '';

  postInstall = ''
    mv zkvms/zkwasm/guest/pkg "$out"/
  '';

  preRunBinaries = [ metacraft-labs.zkwasm ];

  doCheck = false;
})
