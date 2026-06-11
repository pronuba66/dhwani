
# Dhwani

[![Crates.io](https://img.shields.io/crates/v/dhwani.svg)](https://crates.io/crates/dhwani) [![docs.rs](https://docs.rs/dhwani/badge.svg)](https://docs.rs/dhwani/)

**Dhwani** is a real-time node based audio engine for Digital Audio Workstations (DAWs), written in Rust and designed for low-latency sound processing. It focuses on performance, modular DSP building blocks, and efficient I/O for modern audio applications. The engine is built to power synthesizers, mixers, effects chains, and full DAW backends while maintaining Rust’s guarantees for safety and concurrency.

## Meaning

**Dhwani** (Malayalam: ധ്വനി) means **sound**. The word is used across multiple Indian languages including Malayalam, Hindi, Kannada, Telugu, and Marathi.

## Goals

- Real-time safe audio processing
- Modular DSP graph
- Lock-free audio paths
- DAW and synth friendly architecture

## Use cases

- DAW audio engine backend
- Software synthesizers
- Audio effects processors
- Embedded audio systems
- Real-time DSP experimentation

## Folder structure and main files

```text
├── core/              # Dhwani core engine source files
├── core/build_scripts # Library build scripts used by build.rs
├── core/example       # Library example source files
├── core/src           # Library source files
├── daw/               # A miniman DAW GUI application built using Tauri
├── daw/frontend       # Tauri frontend
├── daw/src            # Tauri backend; TS + React source files
├── dhwani.sh          # A helper script file
└── README.md          # This file
```

## Requirements

- cargo
- npm for running DAW GUI application
- tauri cli for npm and cargo (see [Tauri manual setup](https://v2.tauri.app/start/create-project/#manual-setup-tauri-cli))
- run `cd daw/frontend && npm i` before running DAW GUI app

## Example usage

A helper script is provided to run specific target

```text
- `./dhwani.sh test`   # Run tests
- `./dhwani.sh doc`    # Build rust docs
- `./dhwani.sh daw`    # Run minima DAW GUI app
- `./dhwani.sh gui`    # Run a gui example from core crate
- `./dhwani.sh simple` # Run a simpe example from core crate
```

## License

Licensed under either of [Apache License, Version
2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

## Third-Party Assets

### [C Major 9 Bossa Nova Guitar](core/examples/giomilko-c-major-9-bossa-nova-guitar.wav)
- Author: [GioMilko](https://freesound.org/people/GioMilko/)
- Source: https://freesound.org/people/GioMilko/sounds/344211/
- License: CC BY 4.0
