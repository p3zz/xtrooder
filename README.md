## Build
```bash
cargo build --release
```
## Flash & Run
Flashing is performed through probe-run.
First, install it:
```bash
cargo install probe-run
```

Then, flash your binary
```bash
cargo run --bin path/to/bin
```
or 

```bash
cargo run path/to/bin
```