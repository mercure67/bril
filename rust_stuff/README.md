# Bril optimization

This is a tool for optimizing and analyzing Bril programs. It can be built with `cargo build` and can be run as follows:

```sh
# Run in file mode
target/debug/rust_stuff <path/to/program>.json {dce,lvn,df-reaching,df-const}

# Run in pipe mode from a '.bril' file
bril2json < <path/to/program>.bril | target/debug/rust_stuff {dce,lvn,df-reaching,df-const}
```

## Implemented modes

- `dce`: dead code elimination
- `lvn`: local value numbering
- `df-reaching`: reaching definitions dataflow analysis
- `df-const`: constant propagation dataflow analysis

## Testing

### Optimization passes

We use snapshot testing with Turnt, which can be run as follows:

```sh
turnt tests/*/*.bril
```

We also use Brench to test optimizations against all the benchmarks at
`../benchmarks/core/*.bril`, and plot the results using a Python script (dependent on Pandas and Matplotlib).
This can be recreated by building with `cargo build`, then running Brench with:

```sh
brench brench.toml > benchmarks.csv
```

And finally running the Python script with `python plot.py`, which generates a file `benchmarks.png` that should look like this:

![benchmarks](benchmarks.png)

### Dataflow analysis
