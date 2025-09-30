# Documentation

This is the documentation home for the Bril Development Suite.

## Usage

BDS can be built with `cargo build` and run as follows:

```sh
# Run in file mode
target/debug/rust_stuff <path/to/program>.json {dce,lvn,df-reaching,df-const}

# Run in pipe mode from a '.bril' file
bril2json < <path/to/program>.bril | target/debug/rust_stuff {dce,lvn,df-reaching,df-const}
```

## Implemented modes

- [Local optimization](local-opt.md): `dce` for dead code elimination and `lvn` for local value numbering.
- [Dataflow analysis](dfa.md): `df-reaching` for reaching definitions and `df-const` for constant propagation.
- [Global analysis](global.md): `global` for global analysis.
