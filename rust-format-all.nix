{ writeShellApplication, rustfmt, }:
writeShellApplication {
  name = "rustfmt_all";

  runtimeInputs = [ rustfmt ];

  text = ''
    # Using rustfmt instead of cargo fmt, because the latter doesn't support proc-macro crates
    # Additionally, instead of emitting an error, it will "hang", waiting for input
    while read -r file
    do
      rustfmt --edition 2021 -v "$@" "$file" || exit $?
    done <<EOF
    $(find . -type f -name "*.rs" -not -path "*target*")
    EOF
  '';
}
