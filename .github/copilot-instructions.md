# Development Instructions

## Environment

Python is provided via conda. The `LD_LIBRARY_PATH` must be set to the conda
environment's lib directory so the Python shared library is found at link and
test time. This is set up in `.envrc` (used by `direnv`):

```sh
export LD_LIBRARY_PATH=$HOME/.mconda3/envs/opendrift/lib
```

If not using `direnv`, set it manually before running any cargo command:

```sh
export LD_LIBRARY_PATH=$HOME/.mconda3/envs/opendrift/lib
```

## Build

```sh
cargo build
```

## Test

Tests require `LD_LIBRARY_PATH` to be set (see above).

```sh
cargo test -r
```

Or explicitly:

```sh
LD_LIBRARY_PATH=$HOME/.mconda3/envs/opendrift/lib cargo test -r
```

Use release-mode, otherwise geometry tests are very slow.

## Benchmarks

Benchmarks use the `nightly` feature which requires a nightly toolchain and
enables the `test` crate and SIMD support. The nightly toolchain is the
default (see `rustup show`).

```sh
LD_LIBRARY_PATH=$HOME/.mconda3/envs/opendrift/lib cargo bench --features nightly
```
