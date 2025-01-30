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

  text = lib.foldr
    (zkvm: accum: accum + hostPackages."${zkvm}/${guest}" + "/bin/${zkvm}_${guest} \"$@\"\n")
    ""
    zkvms;
}
