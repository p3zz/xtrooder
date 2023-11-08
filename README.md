## Setup
```bash
rustup override set nightly-2023-06-28
```
## Build
```bash
cargo build
```
## Flash & Run
Flashing is performed through probe-run.
First, install it:
```bash
cargo install probe-run
```

Then, flash your binary
```bash
cargo run
```

## Reference
- https://amanjeev.com/blog/stm32-embassy-rust-love/
- https://defmt.ferrous-systems.com/setup.html
- https://docs.rs/defmt-rtt/latest/defmt_rtt/
- https://embassy.dev/
- https://www.st.com/resource/en/user_manual/um2407-stm32h7-nucleo144-boards-mb1364-stmicroelectronics.pdf
- https://ferrous-systems.com/blog/test-embedded-app/