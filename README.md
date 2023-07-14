## Build
```bash
cargo build --release
```

## Flash
```bash
cargo objcopy --release --bin blinky -- -O binary target/thumbv7em-none-eabihf/release/blinky.bin &&
cp target/thumbv7em-none-eabihf/release/blinky.bin /path/to/NODE_H743ZI/
```
