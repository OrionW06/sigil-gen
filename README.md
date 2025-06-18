# sigil-gen

**Rust based sigil generator**

## Overview

`sigil-gen` is a Rust-based application that generates sigils. It leverages the [`macroquad`](https://crates.io/crates/macroquad) crate for graphics and [`chrono`](https://crates.io/crates/chrono) for time handling. This tool is ideal for creating mystical or magical symbols programmatically.

## Features

- Procedural generation of sigils
- Graphical rendering using Macroquad
- Built with Rust for performance and safety

## Installation

Clone the repository and build with Cargo:

```sh
git clone https://github.com/OrionW06/sigil-gen.git
cd sigil-gen
cargo build --release
```

## Usage

Run the generator with:

```sh
cargo run --release
```

This will launch the application, which will display generated sigils in a window.

## Dependencies

- [Rust](https://www.rust-lang.org/) (edition 2021)
- [macroquad](https://crates.io/crates/macroquad)
- [chrono](https://crates.io/crates/chrono)

Dependencies are managed via `Cargo.toml` and will be installed automatically when building.

## Project Structure

- `src/main.rs`: Main entry point and core logic.
- `Cargo.toml`: Dependency and project metadata.

## Contributing

Pull requests and issues are welcome! Please open an issue to discuss your idea or bug before submitting changes.

## License

This project is open source and available under the MIT license.
