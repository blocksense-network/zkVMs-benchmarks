{ writeShellApplication, cargo, }:
writeShellApplication {
  name = "update_nix_dependencies";

  runtimeInputs = [ cargo ];

  text = ''
    updateCrate() {
        sed -i "s|/nix/store/[^-]\+-jolt-[^/]\+|$1|" Cargo.toml
    }

    cd zkvms
    for zkvm in *
    do
        [ ! -d "$zkvm" ] || [ "$zkvm" == 'result' ] && continue
        [ "$zkvm" == 'zkwasm' ] && continue
        pushd "$zkvm"
        newPath="$(nix build github:metacraft-labs/nix-blockchain-development#"$zkvm" --print-out-paths)"

        cd guest
        updateCrate "$newPath"
        cd ../host
        updateCrate "$newPath"
        cd ../wrapper_macro
        updateCrate "$newPath"

        cd ../guest
        cargo generate-lockfile
        cd ../host
        cargo generate-lockfile

        popd
    done
  '';
}
