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
To make *all* zkVMs execute, generate a proof for and verify it run:

```sh
nix run github:blocksense-network/zkVMs-benchmarks#fibonacci
```

Or, to make only [SP1](https://docs.succinct.xyz/docs/sp1/introduction) generate a proof and verify it, you may run:

```sh
nix run github:blocksense-network/zkVMs-benchmarks#sp1/fibonacci -- verify
```

> [!NOTE]
> Take notice of the space between `--` and `verify`!
> It marks an "end of options", as specified by [the POSIX specification](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html#tag_12_02).

As you can tell, you may issue `...#ZKVM/PROGRAM ...` to execute/prove/verify a single program on the chosen zkVMs or `...#PROGRAM ...` to execute, prove *and* verify on *all* zkVMs.
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
The first form executes/proves/verifies the **default** program (currently `graph_coloring`) with the selected zkVM, the second selects a given program to be executed, proved and verified across **all** zkVMs, and with the third you select a specific zkVM and program to act upon.

All command parameters after `--` are passed to it.
As a start, you should look at the built-in help messages.
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
  -r, --runs <REPEAT>
          Benchmark the given action multiple times
  -o, --metrics-output <METRICS_OUTPUT>
          Put the benchmark's formatted output into a file of the given path
  -a, --append
          Append the benchmark formatted output to the given file, instead of
          replacing it
  -h, --help
          Print help
```

```sh
nix run github:blocksense-network/zkVMs-benchmarks#fibonacci -- --help
```

```
A CLI tool for running and benchmarking a guest program inside all supported zkVMs. This bina
ry has been built with a single guest program in mind. If you want to run or benchmark your o
wn guest program inside a zkVM, head on over to https://github.com/blocksense-network/zkVMs-b
enchmarks

Usage: fibonacci [OPTIONS] [ZKVM_ARGS]...

Arguments:
  [ZKVM_ARGS]...  Arguments which are passed to each tool for a single guest and single zkVM

Options:
  -i, --ignore <IGNORE>...  Ignored zkVMs. Values are substrings of names
  -f, --fail-propagation    Make one failiure stop the entire process
  -o, --metrics-output <METRICS_OUTPUT>
          Put the resultant output into a file of the given path
  -a, --append
          Append the resultant output to the given file, instead of replacing it
  -h, --help                Print help
```

### Example: benchmark a single program

As already mentioned, if you omit a zkVM when issuing `nix run`, all zkVMs will be ran for the given program.
However, when benchmarking, to get a useable output, you need to use `--metrics-output`:

```sh
nix run github:blocksense-network/zkVMs-benchmarks#fibonacci -- --metrics-output result.json
```

### Example: benchmark proving of a single program and zkVM with custom input

Contrary to the previous example, with a `ZKVM/PROGRAM` command we need to set a `--benchmark` flag alongside a `--metrics-output` if we want to benchmark (otherwise we just execute the given operation without other output).
We can pass public and private input TOML files as positional arguments, after `prove`:

```sh
nix run github:blocksense-network/zkVMs-benchmarks#sp1/fibonacci -- prove ./private.toml ./public.toml --benchmark --metrics-output result.json
```

Input cannot be fed through stdin and no other format, except TOML, is supported.

## Metrics output format

### `ZKVM/PROGRAM`

When running a `ZKVM/PROGRAM` with the `--benchmark` attribute, additional data is emitted - the metrics output.
This is a simple JSON object, conforming to the following schema:

| Field name    | Type   | Description                                                                    |
| ----------    | ----   | -----------                                                                    |
| timeStarted   | String | Timestamp                                                                      |
| runs          | Number | Positive whole number (greater than 0)                                         |
| totalDuration | Number | How much time the operation took for all runs. Format is seconds.milliseconds  |
| mean          | Number | Average amount of time the operation takes accross all runs                    |
| deviation     | Number | Standard deviation between the durations of all runs                           |
| min           | Number | Shortest duration of the operation across all runs                             |
| max           | Number | Longest duration of the operation across all runs                              |
| memory        | Number | Maximum memory used during the operation in Bytes. **Often null!**             |
| proofSize     | Number | null if no proof was generated, otherwise the size in Bytes. **Often null!**   |

Since this same format is used for `execute`, `prove` and `verify` fields of [`PROGRAM`](#PROGRAM), the last two entries are **not** null **only** when a `PROGRAM` command is ran.

#### Example output

```json
average{
  "timeStarted": "2025-04-29 15:33:42.123863459 +03:00",
  "runs": 3,
  "totalDuration": 16.415178298950195,
  "mean": 5.4717254638671875,
  "deviation": 0.0693691223859787,
  "min": 5.407856464385986,
  "max": 5.545524597167969,
  "memory": null,
  "proofSize": null
}
```

### `PROGRAM`

When running a `PROGRAM` command, another format of the metrics output is emmited.
Again, the format is JSON, conforming to this schema:

| Field name           | Type               | Description                                                                                                                          |
| ------------         | ------             | -----------                                                                                                                          |
| benchmarking         | Array of Benchmark | Stores objects with results and information from a benchmarking operation. New object each time a benchmarking operation is started. |
| hardware             | Hardware Object    | Stores hardware information                                                                                                          |

*Benchmark schema:*

| Field name  | Type   | Description                                                                             |
| ----------  | ----   | -----------                                                                             |
| zkvmName    | String | Name of zkVM, used in the current run                                                   |
| zkvmRev     | String | Commit or tag on which the zkVM is built                                                |
| programName | String | Name of program which is benchmarked                                                    |
| input       | String | Serialized (base64) representation of the input used                                    |
| commit      | String | Commit of the zkVMs-benchmarks repo                                                     |
| execute     | Object | Object of metrics-output form `ZKVM/PROGRAM` or null when the operation isn’t supported |
| prove       | Object | Object of metrics-output form `ZKVM/PROGRAM` or null when the operation isn’t supported |
| verify      | Object | Object of metrics-output form `ZKVM/PROGRAM` or null when the operation isn’t supported |

*Hardware schema:*

| Field name           | Type          | Description                                                       |
| ----------           | ----          | -----------                                                       |
| cpu                  | Array of CPU  | List of unique, dedicated, processors                             |
| memory               | Memory Object | RAM                                                               |
| hardwareAcceleration | Array of GPU  | GPUs **This is always empty!**                                    |
| accelerated          | Boolean       | Whether hardware acceleration was used. **This is always false!** |

*CPU schema:*

| Field name | Type   | Description                            |
| ---------- | ----   | -----------                            |
| model      | String | CPU model                              |
| cores      | Number | Positive whole number (greater than 0) |
| speed      | Number | CPU speed (in GHz)                     |

*Memory schema:*

| Field name | Type   | Description           |
| ---------- | ----   | -----------           |
| model      | String | Memory model          |
| size       | Number | Memory size (in Bytes). **Available only when command is ran with root permissions!**   |
| speed      | Number | Memory speed (in MHz) **Available only when command is ran with root permissions!**|

*GPU schema:*

| Field name | Type   | Description    |
| ---------- | ----   | -----------    |
| model      | String | GPU model      |
| cores      | Number | GPU cores      |
| speed      | Number | GPU core speed |

#### Example output

**The command which produced this output was ran with root permissions!**

```json
{
  "benchmarking": [
    {
      "name": "/nix/store/lbp252948zkvlansp5x40y8sxhh2nagk-sp1_fibonacci-unstable-2025-03-10/bin/sp1_fibonacci",
      "execute": {
        "timeStarted": "2025-04-29 15:19:17.343992869 +03:00",
        "runs": 1,
        "totalDuration": 0.0038770779501646757,
        "mean": 0.0038770779501646757,
        "deviation": 0,
        "min": 0.0038770779501646757,
        "max": 0.0038770779501646757,
        "memory": 1125048320,
        "proofSize": 192
      },
      "prove": {
        "timeStarted": "2025-04-29 15:19:19.119432201 +03:00",
        "runs": 1,
        "totalDuration": 5.817917346954346,
        "mean": 5.817917346954346,
        "deviation": 0,
        "min": 5.817917346954346,
        "max": 5.817917346954346,
        "memory": 4881145856,
        "proofSize": 192
      },
      "verify": {
        "timeStarted": "2025-04-29 15:19:32.385317873 +03:00",
        "runs": 1,
        "totalDuration": 0.10108701884746552,
        "mean": 0.10108701884746552,
        "deviation": 0,
        "min": 0.10108701884746552,
        "max": 0.10108701884746552,
        "memory": 4787023872,
        "proofSize": 192
      }
    },
  ],
  "hardware": {
    "cpu": [
      { "model": "AMD Ryzen 9 9950X 16-Core Processor", "cores": 16, "speed": 600 }
    ],
    "memory": {
      "model": "KF556C36-32",
      "size": 132511961088,
      "speed": 4800
    },
    "hardwareAcceleration": [],
    "accelerated": false
  }
}
```

## Limitations

Due to the complicated ways in which Nix ([craneLib](https://crane.dev/)) and cargo interact, most of the packages in this repository do **not** compile without Nix.
This also means that incremental/debug builds are not really possible.

As of writing, user-defined input types (argument types to the entrypoint function) are not supported.
This also includes types defined by any libraries you may use.
