# Performance

Pleiades uses a production release profile with fat LTO, one code-generation unit, symbol stripping, and abort-on-panic. These settings favor startup speed and a compact distributable binary at the cost of a slower release build.

Model discovery requests are issued concurrently, so startup paths that need multiple provider catalogs wait for the slowest provider rather than the sum of every provider latency. Built-in prompt parsing is cached for the process lifetime, and message text assembly avoids cloning every content block.

Run the repeatable microbenchmarks with:

```console
cargo bench --workspace
```

For release artifact measurements:

```console
cargo build --release --bin pleiades
du -h target/release/pleiades
/usr/bin/time -v target/release/pleiades --help
```

Record compiler version, operating system, CPU, binary size, elapsed time, and peak resident memory when comparing results. CI runs Criterion benchmarks on changes to Rust sources.

## Baseline

The first optimized build measured on Linux x86-64 with Rust 1.88 produced a 6,270,960-byte binary. A cold `pleiades --help` invocation reported under 0.01 seconds elapsed and approximately 6.4 MB peak resident memory. Treat these figures as a local baseline rather than cross-platform guarantees.
