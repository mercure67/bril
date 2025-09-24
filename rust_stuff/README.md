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

#### Reaching definitions

We tested our implementation of the DFA algorithm for reaching definitions on two programs: `little.bril` (the diamond example from lecture), and `rec.bril` (a small recursive factorial implementation).

Here's the output of `bril2json < tests/reaching/little.bril | target/debug/rust_stuff df-reaching`:

```json
block: f0.b0
in: []
out: [Defn { name: "fmainval0", var: "a" }, Defn { name: "fmainvbl1", var: "b" }, Defn { name: "fmainvcondl0", var: "cond" }]

block: f0.b1
in: [Defn { name: "fmainval0", var: "a" }, Defn { name: "fmainvbl1", var: "b" }, Defn { name: "fmainvcondl0", var: "cond" }]
out: [Defn { name: "fmainval0", var: "a" }, Defn { name: "fmainvbl4", var: "b" }, Defn { name: "fmainvcl5", var: "c" }, Defn { name: "fmainvcondl0", var: "cond" }]

block: f0.b2
in: [Defn { name: "fmainval0", var: "a" }, Defn { name: "fmainvbl1", var: "b" }, Defn { name: "fmainvcondl0", var: "cond" }]
out: [Defn { name: "fmainval8", var: "a" }, Defn { name: "fmainvbl1", var: "b" }, Defn { name: "fmainvcl9", var: "c" }, Defn { name: "fmainvcondl0", var: "cond" }]

block: f0.b3
in: [Defn { name: "fmainval0", var: "a" }, Defn { name: "fmainval8", var: "a" }, Defn { name: "fmainvbl1", var: "b" }, Defn { name: "fmainvbl4", var: "b" }, Defn { name: "fmainvcl5", var: "c" }, Defn { name: "fmainvcl9", var: "c" }, Defn { name: "fmainvcondl0", var: "cond" }]
out: [Defn { name: "fmainval0", var: "a" }, Defn { name: "fmainval8", var: "a" }, Defn { name: "fmainvbl1", var: "b" }, Defn { name: "fmainvbl4", var: "b" }, Defn { name: "fmainvcl5", var: "c" }, Defn { name: "fmainvcl9", var: "c" }, Defn { name: "fmainvcondl0", var: "cond" }, Defn { name: "fmainvdl12", var: "d" }]
```

This output, which shows the outgoing reaching definitions for each block, matches precisely the results from lecture: on either side block, different definitions of `a`, `b`, and `c` reach. In particular, we notice how the left-hand path (through block 1) overwrites `b` and defines `c`, whereas the right-hand path (through block 2) overwrites `a` and defines `c`. Thus all six definitions of `a`, `b`, and `c` reach the end of the program, as well as the unchanged `cond`, and `d`, which is defined in block 3 (bottom block).

![little.bril](tests/reaching/little.png)

Now consider the output for the recursive factorial program, `rec.bril`:

```json
block: f0.b0
in: []
out: [Defn { name: "ffactvcondl3", var: "cond" }, Defn { name: "ffactvnl0", var: "n" }, Defn { name: "ffactvonel2", var: "one" }, Defn { name: "ffactvzerol1", var: "zero" }]

block: f0.b1
in: [Defn { name: "ffactvcondl3", var: "cond" }, Defn { name: "ffactvnl0", var: "n" }, Defn { name: "ffactvonel2", var: "one" }, Defn { name: "ffactvzerol1", var: "zero" }]
out: [Defn { name: "ffactvcondl3", var: "cond" }, Defn { name: "ffactvnl0", var: "n" }, Defn { name: "ffactvonel2", var: "one" }, Defn { name: "ffactvzerol1", var: "zero" }]

block: f0.b2
in: [Defn { name: "ffactvcondl3", var: "cond" }, Defn { name: "ffactvnl0", var: "n" }, Defn { name: "ffactvonel2", var: "one" }, Defn { name: "ffactvzerol1", var: "zero" }]
out: [Defn { name: "ffactvcondl3", var: "cond" }, Defn { name: "ffactvn1l8", var: "n1" }, Defn { name: "ffactvnl0", var: "n" }, Defn { name: "ffactvonel2", var: "one" }, Defn { name: "ffactvtmpl9", var: "tmp" }, Defn { name: "ffactvzerol1", var: "zero" }]

block: f0.b3
in: [Defn { name: "ffactvcondl3", var: "cond" }, Defn { name: "ffactvn1l8", var: "n1" }, Defn { name: "ffactvnl0", var: "n" }, Defn { name: "ffactvonel2", var: "one" }, Defn { name: "ffactvtmpl9", var: "tmp" }, Defn { name: "ffactvzerol1", var: "zero" }]
out: [Defn { name: "ffactvcondl3", var: "cond" }, Defn { name: "ffactvn1l8", var: "n1" }, Defn { name: "ffactvnl0", var: "n" }, Defn { name: "ffactvonel2", var: "one" }, Defn { name: "ffactvresultl10", var: "result" }, Defn { name: "ffactvtmpl9", var: "tmp" }, Defn { name: "ffactvzerol1", var: "zero" }]

block: f1.b0
in: []
out: [Defn { name: "fmainvnl0", var: "n" }, Defn { name: "fmainvresl1", var: "res" }]

block: f1.b1
in: [Defn { name: "fmainvnl0", var: "n" }, Defn { name: "fmainvresl1", var: "res" }]
out: [Defn { name: "fmainvnl0", var: "n" }, Defn { name: "fmainvresl1", var: "res" }]
```

Unfortunately, this output does not match our expectations. We posit that the CFG generator is yielding an incorrect output, as it does not manage interprocedural jumps well (related to [this Zulip thread](https://cs6120.zulipchat.com/#narrow/channel/254729-general/topic/.E2.9C.94.20Why.20isn't.20.60call.60.20a.20terminator.20for.20basic.20blocks.3F/with/496384549)). We also theorize that the worklist implementation and both 

#### Constant propagation

Refer to the [constant propagation testing document](constprop.md).
