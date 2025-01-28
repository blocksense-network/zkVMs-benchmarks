{ zkVM-helpers,
  lib,
  rust-bin,
  metacraft-labs,
  wasm-pack,
  wasm-bindgen-cli,
  binaryen,
  craneLib-default,
}:
let
  commonArgs = {
    pname = "zkwasm";
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
  };

  rust-toolchain = rust-bin.nightly."2024-04-09".default.override {
    targets = ["wasm32-unknown-unknown"];
  };
  craneLib = craneLib-default.overrideToolchain rust-toolchain;
  cargoArtifacts = craneLib.buildDepsOnly (zkVM-helpers.fixDeps commonArgs);
in
  craneLib.buildPackage (zkVM-helpers.withCustomPhases (commonArgs
    // {
      inherit cargoArtifacts;

      nativeBuildInputs = [
        metacraft-labs.zkwasm
        wasm-pack
        wasm-bindgen-cli
        binaryen
      ];

      postPatch = ''
        ln -s ../../../Cargo.lock ./zkvms/zkwasm/guest/
      '';

      preBuildGuest = ''
        # Workaround from
        # https://github.com/rustwasm/wasm-pack/issues/1335
        export WASM_PACK_CACHE=.wasm-pack-cache
      '';

      buildGuestCommand = "wasm-pack build --release --frozen";

      preBuild = ''
        export GUEST_PATH="$out/pkg/guest_bg.wasm"
      '';

      postInstall = ''
        mv zkvms/zkwasm/guest/pkg "$out"/
      '';

      preRunBinaries = [
        metacraft-labs.zkwasm
      ];

      doCheck = false;
    }))
