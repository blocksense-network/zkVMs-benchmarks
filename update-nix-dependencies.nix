{ writeShellApplication, cargo, }:
writeShellApplication {
  name = "update_nix_dependencies";

  runtimeInputs = [ cargo ];

  text = ''
    updateCrate() {
        sed -i "s|/nix/store/[^-]\+-$1-[^/]\+|$2|" Cargo.toml
    }

    cd zkvms
    for zkvm in *
    do
        [ ! -d "$zkvm" ] || [ "$zkvm" == 'result' ] && continue
        [ "$zkvm" == 'zkwasm' ] && continue
        pushd "$zkvm"
        newPath="$(nix build github:metacraft-labs/nix-blockchain-development#"$zkvm" --print-out-paths)"

        [ "$zkvm" == 'nexus' ] && zkvm=Nexus

        cd guest
        updateCrate "$zkvm" "$newPath"
        cd ../host
        updateCrate "$zkvm" "$newPath"
        cd ../wrapper_macro
        updateCrate "$zkvm" "$newPath"

        cd ../guest
        cargo generate-lockfile
        cd ../host
        cargo generate-lockfile

        popd
    done
  '';
}
