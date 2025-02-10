This is a copy of [ZKM's project SDK](https://github.com/zkMIPS/zkm-project-template/tree/main/sdk), which changes the zkVM cargo dependencies to point to `/nix/store`.
It is also used as a convinent place which stores the SDK's `libsnark` (`sdk/src/local/libsnark`), so during compilation it is not fetched from the source repo (refer to `zkvms/zkm/default.nix`).

---

# ZKM SDK usage

## Use the libsnark

1. The  compile.sh in the path sdk/src/local/libsnark only supports X86_64 linux.For MacOS, there is a [Dockerfile](../Dockerfile) in the template.
   
```
cd zkm-project-template/sdk/src/local/libsnark
./compile.sh
```
    If successful, it will generate the libsnark.so in sdk/src/local/libsnark/

2. To instruct your Rust environment on the location of the libsnark.so , you can set the LD_LIBRARY_PATH environment variable. For example:

```
export LD_LIBRARY_PATH=Your BASEDIR/zkm-project-template/sdk/src/local/libsnark:$LD_LIBRARY_PATH  
```

3. Import the SDK
   
```
// Cargo.toml
[dependencies]
zkm-sdk = { git = "https://github.com/zkMIPS/zkm-project-template", branch = "main", features = ["snark"] }
```

## Don't use the libsnark

1. Set the environment variable `NO_USE_SNARK=true` .
  
2. Import the SDK
   
```
// Cargo.toml
[dependencies]
zkm-sdk = { git = "https://github.com/zkMIPS/zkm-project-template", branch = "main" }
```
