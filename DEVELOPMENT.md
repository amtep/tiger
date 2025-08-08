## Benchmarks

First, create config files in `benches/`. Copy the `.example` files and adjust the paths for your system.

Whole system benchmarks are run with:
```
cargo bench -p ck3-tiger
```
They support baselines with:
```
cargo bench -p ck3-tiger -- --save-baseline my_baseline
cargo bench -p ck3-tiger -- --baseline my_baseline
```

Internal benchmarks are run with:
```
cargo bench --no-default-features --features="ck3,internal_benches"
```
They do not support baselines
