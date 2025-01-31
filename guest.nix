{ writeShellApplication,
  guest,
  zkvms,
  hostPackages,
  lib,
}:
writeShellApplication {
  name = "${guest}";

  runtimeInputs = lib.foldr
    (zkvm: accum: accum ++ [ hostPackages."${zkvm}/${guest}" ])
    []
    zkvms;

  text = ''
    runZKVM() {
      echo "$1"
      "$@"
    }
  '' + lib.foldr
    (zkvm: accum: accum + "runZKVM \"${hostPackages."${zkvm}/${guest}"}/bin/${zkvm}_${guest}\" \"$@\"\n")
    ""
    zkvms;
}
