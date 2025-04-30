{ craneLib-default, guest, zkvms, hostPackages, lib, benchexec, }:
let
  commonArgs = {
    name = "${guest}";

    buildInputs = [ benchexec ] ++
      (lib.foldr (zkvm: accum: accum ++ [ hostPackages."${zkvm}/${guest}" ]) [ ]
      zkvms);

    src = lib.fileset.toSource {
      root = ./.;
      fileset = ./.;
    };

    PROGRAMS = lib.foldr (zkvm: accum:
      hostPackages."${zkvm}/${guest}" + "/bin/${zkvm}_${guest}," + accum) ""
      zkvms;

    postPatch = ''
      sed -i 's|"runexec"|"${benchexec}/bin/runexec"|' ./src/main.rs
    '';
  };

  cargoArtifacts = craneLib-default.buildDepsOnly commonArgs;
in craneLib-default.buildPackage commonArgs
