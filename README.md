# Xtrooder - An embedded firmware for 3D printers, written in Rust

## Description
An async firmare for 3D printers built on top of the [embassy](https://github.com/embassy-rs/embassy) framework. The project is part of a master thesis in embedded systems at the [University of Trento](https://www.unitn.it/). If you want to dive deep into the project and the theory behind it, you can check the *docs/thesis.pdf* file. The following lines will give the user a minimal knowledge on how to built and run the project.

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
cd board
cargo run

# release mode
cd board
cargo run --release

# defmt-log feature
cd board
cargo run --features defmt-log
```

## Test
The *host* workspace provides a huge number of test-suites, one for each sub-crate.
To run the tests, run:
```bash
cd host
cargo test
```
Some tests can take a while, in particular for the async ones, which are run using the [tokio](https://github.com/tokio-rs/tokio) runtime.

## Notes
- A logging features is provided by the [defmt](https://github.com/knurling-rs/defmt) crate
- The final finary file can be huge if built in debug mode (close to 2MB). If you have flash memory restriction,
it is recommended to built the project in release mode to achieve a binary file close to 200Kb in size.

## Contribution
If you want to collaborate, hit me on federico.pezzato.work@gmail.com.