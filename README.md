# Xtrooder - An embedded firmware for 3D printers, written in Rust

## Description
A firmare for 3D printers built on top of the [embassy](https://github.com/embassy-rs/embassy) framework.

## Structure
The project is divided in 2 separate workspaces:
- board: contains code that is strictly related to the MCU the user is going to use
- host: contains code that can be compiled both on host and board 

## Build
Each workspace can be built separately, but the whole project can be built through the **board** workspace.

### Board
The **board** workspace needs a nightly toolchain, which is specified in its rust-toolchain.toml.
For now, only *thumbv7em-none-eabihf* has been tested has target (using a [Nucleo-H753zi](https://www.st.com/en/evaluation-tools/nucleo-h753zi.html)).
To build the workspace, run:
```bash
cd board
cargo build
```

### Host
The **host** workspace can be built with the stable toolchain (only tested for linux-x86).
To build the workspace, run:
```bash
cd board
cargo build
```

## Run
The *run* configuration is provided by the .cargo/config.toml file of the *board* workspace.
The user will need [probe-rs](https://github.com/probe-rs/probe-rs) to flash and load the binary file.
To flash and run the project, run:
```bash
# debug mode
cargo run

# release mode
cargo run --release

# defmt-log feature
cargo run --features defmt-log
```

## Notes
- A logging features is provided by the [defmt](https://github.com/knurling-rs/defmt) crate
- The final finary file can be huge if built in debug mode (close to 2Mb). If you have flash memory restriction,
it is recommended to built the project in release mode to achieve a binary file close to 200Kb in size.

## Contribution
The project is part of a master thesis in embedded systems. If you want to dive deep into the project and the theory behind it, you can check the *docs/thesis.pdf* file.