# Bril Property Analysis

This is a set of tools that analyze properties of a Bril program. It can be run using `cargo` as follows:

```sh
cargo run -- <mode> <path/to/program>.json
```

## Implemented modes

- `varmap` (variable mapping): prints a map between function names and sets of variables used in each function.

## Testing

We use snapshot testing with Turnt. The tests can be run as follows:

```sh
turnt tests/*.bril
```
