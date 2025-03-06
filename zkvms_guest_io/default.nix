{ craneLib-default, guest, zkvms, hostPackages, lib, }:
let
  commonArgs = {
    name = "${guest}";

    buildInputs =
      lib.foldr (zkvm: accum: accum ++ [ hostPackages."${zkvm}/${guest}" ]) [ ]
      zkvms;

    src = lib.fileset.toSource {
      root = ./.;
      fileset = ./.;
    };

    PROGRAMS = lib.foldr (zkvm: accum:
      hostPackages."${zkvm}/${guest}" + "/bin/${zkvm}_${guest}," + accum) ""
      zkvms;
  };

  cargoArtifacts = craneLib-default.buildDepsOnly commonArgs;
in craneLib-default.buildPackage commonArgs
