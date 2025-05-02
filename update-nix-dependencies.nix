{ writeShellApplication, cargo, zkvms, }:
writeShellApplication {
  name = "update_nix_dependencies";

  runtimeInputs = [ cargo ];

  text = let
    namesAndPaths = builtins.concatStringsSep " "
      (builtins.map (zkvm: zkvm.pname + "," + zkvm.outPath) zkvms);
  in ''
    updatePath() {
        sed -i "s|/nix/store/[^-]\+-$1-[^/]\+|$2|" Cargo.toml
    }
    updateDep() {
        updatePath "$1" "$2"
        cargo generate-lockfile
    }

    cd zkvms
    for i in ${namesAndPaths}
    do
        IFS=',' read -r zkvm path <<< "''${i}"
        [ "$zkvm" == 'zkWasm' ] && continue
        [ "$zkvm" == 'Nexus-zkVM' ] && zkvm=nexus

        pushd "$zkvm"

        [ "$zkvm" == 'nexus' ] && zkvm=Nexus

        cd wrapper_macro
        updatePath "$zkvm" "$path"
        cd ../guest
        updateDep "$zkvm" "$path"
        cd ../host
        updateDep "$zkvm" "$path"

        popd
    done
  '';
}
