## Build
```bash
cargo build --release
```

## Flash
```bash
cargo objcopy --release --bin printer -- -O binary target/thumbv7em-none-eabihf/release/printer.bin
cp target/thumbv7em-none-eabihf/release/printer.bin /path/to/NODE_H743ZI/
```
