## Build
```bash
cargo build --release
```

## Flash (don't use it, WIP)
```bash
cargo objcopy --release --bin printer -- -O binary target/thumbv7em-none-eabihf/release/printer.bin
cp target/thumbv7em-none-eabihf/release/printer.bin /path/to/NODE_H743ZI/
```

## Flash (use this instead)
The operation is performed through an openocd server and gdb-multiarch

Open an OpenOCD server:
```bash
openocd
```

Connect to it through gdb-multiarch, specifying the file you want to flash:
```bash
gdb-multiarch -x openocd.gdb <executable>
```