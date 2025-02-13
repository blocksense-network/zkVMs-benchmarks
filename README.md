# zkVMs-benchmarks

This is a repository with setups and programs for zero-knowledge virtual machine benchmarking.
Its ultimate goal is to deliver reproducible and accurate performance metrics across many zkVMs, so **you** can choose which technology suits your needs!

Being made with reproducibility in mind, this project also serves as a good framework for running programs across zkVMs without the complicated and ever-changing setups required to do so.

## Usage

The backbones of this entire codebase are [Nix](https://nixos.org/) and Linux.
MacOS is not supported!
Windows is supported via WSL.

**First**, install the Nix package manager, follow their [download instructions](https://nixos.org/download/).
Generally it should be enough to:

```sh
sh <(curl -L https://nixos.org/nix/install) --daemon
```

> [!WARNING]
> It is preferable to use the nixos.org script, as shown above!
> Certain systems provide Nix with their native package managers, however practice has shown those do not always lead to working setups!

Now, what follows depends on your use case.

### Run/benchmark a "built-in" program

The `guests` directory provides a variety of programs, all of which are proven to work with their default inputs.
It is advisable to first try and run one of them, so you can make sure Nix is installed and working properly.

The smallest one is `fibonacci`.
To make *all* zkVMs generate a proof for it just run:

```sh
nix run github:blocksense-network/zkVMs-benchmarks#fibonacci -- prove
```

> [!NOTE]
> Take notice of the space between `--` and `prove`!
> It marks an "end of options", as specified by [the POSIX specification](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html#tag_12_02).

Or to make, for example, [SP1](https://docs.succinct.xyz/docs/sp1/introduction) generate a proof and verify it, you may run:

```sh
nix run github:blocksense-network/zkVMs-benchmarks#sp1/fibonacci -- verify
```

As you can tell, you may issue `...#ZKVM/PROGRAM ...` to execute/prove/verify a single program on the chosen zkVMs or `...#PROGRAM ...` to do the same on *all* zkVMs.
The format `...#ZKVM ...` is also supported, pairing the chosen zkVM with the **default** guest program (currently `graph_coloring`).

The possible values for `ZKVM` correspond to the directory names inside `zkvms` folder at the root of the repository.
Consequently, `PROGRAM` values are connected to directory names inside `guests`.

### Run/benchmark your own program

1. Clone the [git repository](https://github.com/blocksense-network/zkVMs-benchmarks)

   ```sh
   git clone git@github.com:blocksense-network/zkVMs-benchmarks.git
   ```

2. Navigate to the `guests` directory

   ```sh
   cd zkVMs-benchmarks/guests
   ```

3. Follow the instructions inside the `guests/README.md` file to setup your program

  > [!NOTE]
  > Remember to `git add` your project!
  > Nix only sees files which are tracked by git.

4. Use the `.` path as the `nix run` source.
   So, for example, if you want to create a proof for your program with [Jolt](https://jolt.a16zcrypto.com/), you can run:

   ```sh
   nix run .#jolt/NAME -- prove
   ```

   Where `NAME` is the name of your program (inside `guests`).

## Command arguments

The general format for `nix run` is:

```sh
nix run github:blocksense-network/zkVMs-benchmarks#BINARY -- PARAMETERS
```

`BINARY` is either in the form `ZKVM`, `PROGRAM` or `ZKVM/PROGRAM`.

As already discussed in "[Run/benchmark a "built-in" program](#runbenchmark-a-built-in-program)", the possible values for `ZKVM` are the subdirectory names inside `zkvms` and for `PROGRAM` are the subdirectory names inside `guests`.
The first form executes/proves/verifies the **default** program (currently `graph_coloring`) with the selected zkVM, the second selects a given program to be ran across **all** zkVMs and the third chooses a specific zkVM and program to act upon.

All command parameters after `--` are passed to it.
As a start, you should look at the built-in help message.
Further in this section there are some common configurations of arguments you may want to use.

```sh
nix run github:blocksense-network/zkVMs-benchmarks#sp1/fibonacci -- --help
```

```
A CLI tool for running and benchmarking guest programs inside a zkVM environment.
This binary has been built with a single zkVM and guest program in mind. If you
want to run or benchmark your own guest program inside a zkVM, head on over to
https://github.com/blocksense-network/zkVMs-benchmarks

Usage: host-sp1 [OPTIONS] <RUN_TYPE> [PRIVATE_INPUT] [PUBLIC_INPUT]

Arguments:
  <RUN_TYPE>       What should the zkVM do with the guest
                   [possible values: execute, prove, verify]
  [PRIVATE_INPUT]  Path to private input file (in TOML format)
  [PUBLIC_INPUT]   Path to public input file (in TOML format)

Options:
  -b, --benchmark
          Enable benchmark timer and formatted output
  -r, --repeat <REPEAT>
          Benchmark the given action multiple times
  -m, --millis
          Output timings as milliseconds instead of seconds
  -o, --metrics-output <METRICS_OUTPUT>
          Put the benchmark's formatted output into a file of the given path
  -a, --append
          Append the benchmark formatted output to the given file, instead of
          replacing it
  -h, --help
          Print help
```

### Example: benchmark a single program

As already mentioned, if you omit a zkVM when issuing `nix run`, all zkVMs will be ran for the given program.
However, when benchmarking, to get a useable output, you need to use `--metrics-output` with `--append`:

```sh
nix run github:blocksense-network/zkVMs-benchmarks#fibonacci -- prove --benchmark --metrics-output result.csv --append
```

### Example: benchmark a single program with custom input and millisecond precision

Extending on the previous example, we can pass public and private input TOML files as positional arguments, after `prove`:

```sh
nix run github:blocksense-network/zkVMs-benchmarks#fibonacci -- prove ./private.toml ./public.toml -bamo result.csv
```

Input cannot be fed through stdin and TOML is the only supported format.

## Benchmark metrics output format

When running with the `--benchmark` attribute, additional data is emitted - the metrics output.
This is a very simple CSV content, with two columns: the first a name and the second a value.

Here is a table with the currently available pairs:

| Name     | Value type | Shown  |
| -------- | ---------- | ------ |
| zkvm     | String     | Always |
| guest    | String     | Always |
| duration | Integer    | Always |
| repeats  | Integer    | Always |
| average  | Integer    | Always |

### Example output

```csv
zkvm,sp1
guest,fibonacci
duration,4
repeats,1
average,4
```

## Limitations

Due to the complicated ways in which Nix ([craneLib](https://crane.dev/)) and cargo interact, most of the packages in this repository do **not** compile without Nix.
This also means that incremental/debug builds are not really possible.

As of writing, user-defined input types (argument types to the entrypoint function) are not supported.
This also includes types defined by any libraries you may use.
