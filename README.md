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

## Reference
- https://amanjeev.com/blog/stm32-embassy-rust-love/
- https://defmt.ferrous-systems.com/setup.html
- https://docs.rs/defmt-rtt/latest/defmt_rtt/
- https://embassy.dev/
- https://www.st.com/resource/en/user_manual/um2407-stm32h7-nucleo144-boards-mb1364-stmicroelectronics.pdf