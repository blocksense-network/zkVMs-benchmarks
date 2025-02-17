# Guest programs

This directory contains all programs which can be executed/proven/verified by a zkVM.
These are normal Rust programs, with certain specific patterns implemented for zkVM compatibility.

## Adding your own program

> [!IMPORTANT]
> **Fully implement and test your program before trying to add it here!**
>  Although this repo makes it easy to run your program on all zkVMs, developing is not made easier.

1. **Copy the project to this directory.**

   If you refer to local path dependencies, which you want to include with the project, move them inside the `guest/YOUR_PROJECT/` directory!
   Everything directly inside `guests` is expected to be a crate, setup to be ran by the zkVMs.

2. **Convert your project into a library.**

   You'll mainly need to rename `src/main.rs` to `src/lib.rs`.

3. **Update your `Cargo.toml`**

   Add the `guests_macro` [dependency](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) **exactly like this**:

   ```toml
   guests_macro = { version = "0.1.0", path = "../../guests_macro" }
   ```

   and add a `no_std` [feature](https://doc.rust-lang.org/cargo/reference/features.html) like this:

   ```toml
   no_std = []
   ```

   It's ok if you don't conditionally include anything with the feature, however it **must** exist!

4. **Update your program.**

   1. Above your main (entrypoint) function add a `#[guests_macro::proving_entrypoint]` attribute
   2. Make sure the function is public
   3. Move all input variable declarations as arguments to the main function and remove their assignments.
      Input parsing is built-in and automagically handles types.
   4. In case your program works without the standard library, or you want to support lack of std, remember to add  
      `#![cfg_attr(feature = "no_std", no_std)]` at the top of your `lib.rs`

   So, if you have something like:

   ```rust
   fn main() {
       let n: u8 = read!();
       ...
   }
   ```

   transform it to:

   ```rust
   #[guests_macro::proving_entrypoint]
   pub fn main(n: u8) {
       ...
   }
   ```

   The entrypoint function does not need to be called `main` (the project is now a library).

5. **Add or update a `Cargo.lock`.**

   Using any version of cargo, you simply need to do:

   ```sh
   cargo generate-lockfile
   ```

6. **Add the "default" files.**

   Three additional files **must** exist in your `guests/YOUR_PROJECT/` directory, containing default values: `default.env`, `default_private_input.toml`, `default_public_input.toml`.
   Depending on your situation, it may be admissible to have any of these empty, but they **must** exist!

   1. The first, `default.env`, contains pairs of `NAME=VALUE` and line comments, similar to shell scripts.
      These values set zkVM-specific options.
      Information on the possible values, their meaning and how to choose them, can be found [here](../zkvms/README.md#zkvm-specific-environment-variables).
      However, generally, you'll only need to set ZKM's `SEG_SIZE`.

      A simple example comes from `fibonacci`:

      ```sh
      # ZKM
      SEG_SIZE=2783
      ```

   2. `default_private_input.toml` and `default_public_input.toml` contain default input values for your program.
      Each [key](https://toml.io/en/v1.0.0#keyvalue-pair) is the name of an attribute in your main function.
      Keys between the two files must be unique, meaning each main function attribute is defined in **only one** of the files.

      Whether an input is public or not is defined in these files, **not** in your code!
      It is preferable for your default input to be short, so the default execution, proving and verification steps are fast.

      Again, simple examples are found in `fibonacci`.
      For the following main function:

      ```rust
      /* ... */
      pub fn main(n: u8, fN: u64) -> bool {
      /* ... */
      ```

      this default private input:

      ```toml
      fN = 259695496911122585
      ```

      and this default public input:

      ```toml
      n = 85
      ```

      are correct.

7. **Track or commit your project with git.**

   Due to the way Nix works, you'll need to at least track your guest program (but you probably should commit it).

   ```sh
   git add guests/YOUR_PROJECT
   ```

## Using a program

You may execute/prove/verify a program in this directory (when the repository is cloned) by issuing:

```sh
nix run .#YOUR_PROJECT prove
```
