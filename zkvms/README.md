# Host programs and guest wrapper

This directory contains all setups for all supported zkVMs.

## Host-guest architecture

For each zkVM, we define a host program: this program uses the zkVM as a library, initializing the required objects, starting the execute/prove/verify process and feeding the user program it's required input.
Input and output (command-line options) are handled by `zkvms_host_io` crate, in the root of the repository.

Guest programs are our user programs, the ones inside `guests`, which are executed/proven/verified by the vm.

It is important to note that usually there are exceptions to this architecture, however, here everything is homogenised.

## File structure

In each zkVM folder you'll find three subdirectories: `guest`, `host` and `wrapper_macro`.

To make input and output handling generic, we define a guest "wrapper" crate inside `guest`, which reads all input with the zkVM-specific functions, converts types if necessary, then calls the main (entrypoint) function of our actual guest program and finally outputs (commits) the results with the zkVM-specific functions.

We need to be extra generic in our guest wrapper, because the type of our input function (and therefore, our information on what we need to do) exists only in the user's guest implementation.
That is the reason why every guest must include a `#[guests_macro::proving_entrypoint]` attribute above their main function: it extracts that type and converts it into an "object" which can then be used inside the wrapper.

That is also the reason why reading input and outputting the result is handled by `wrapper_macro`.

> [!NOTE]
> You may be tempted to think about using Rust generics, but in our case we can have:
>
> 1. Arbitrary types
> 2. Arbitrary amount of types
> 3. Arbitrary handling of types
>
> In certain cases, the zkVM input function returns all arguments, in others it returns every argument one by one, in third cases it returns only a primitive value and we add logic to "reconstruct" the actual type.
> Using macros is the simplest and most effective way to go.

`host` contains the host implementation, as is explained in the previous section.
That is, it initializes the zkVM-specific set of object with the proper parameters, configures our input and then passes our guest wrapper as the program to execute/prove/verify.

## zkVM-specific environment variables

Options which are universal to all zkVMs are defined as command-line flags in `zkvms_host_io`.
However, zkVM-specific configuration options are defined as environment variables.

The value for each of these is taken, with this precedence, by the user shell environment, if it is set, then by the guest implementation's `default.env` if it exists, and finally as a pre-built constant if all else fails.

| zkVM   | Variable name      | Type of value | Built-in default | Description         |
| ------ | ------------------ | ------------- | ---------------- | ------------------- |
| zkm    | SEG_SIZE           | integer       | 65536            | The segment size. You'll need to find this value experimentally, as it cannot be too low or too high for your specific guest implementation. |
|        | PROOF_RESULTS_PATH | path (string) | /tmp/contracts   | Directory to output the proof (files)                                                                                                        |
|        | VERIFYING_KEY_PATH | path (string) | /tmp/input       | Directory to output the verification key                                                                                                     |
| zkwasm | ZKWASM_K           | integer       | 19               | The "K" value for zkWasm. It's value is between 19 and 22 inclusive. For larger inputs you'll need to increment it, but the proving time also increases. You'll need to find the smallest one experimentally.|
|        | ZKWASM_SCHEME      | string        | shplonk          | Proving system                                                                                                                               |
|        | ZKWASM_OUTPUT      | path (string) | ./output         | Directory to store output                                                                                                                    |
|        | ZKWASM_PARAMS      | path (string) | ./params         | Directory to store parameter information                                                                                                     |

> [!NOTE]
> Looking through the source code, you may notice other environment variables which are used.
> Those are setup by Nix during compilation, so their overriding is not officially supported.
