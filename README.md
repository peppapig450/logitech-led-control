# Logitech LED Control

A command-line utility written in Rust for controlling the lighting of supported Logitech keyboards. It allows listing attached devices, setting key colors or groups, applying effects, and loading profiles.

## Features

- Detect and open Logitech keyboards via HID or libusb backend.
- Set colors for individual keys, groups, or regions.
- Apply and store native lighting effects.
- Configure startup and onboard modes.
- Load lighting profiles from files or standard input.

## Building

This project uses Cargo and requires the stable Rust toolchain. Run the following to build and test:

```bash
cargo build
cargo test
```

By default the HID backend is used. To enable the libusb backend, compile with the `libusb` feature:

```bash
cargo build --features libusb
```

## Usage

List all connected keyboards:

```bash
logi-led list-keyboards
```

Set all keys to red without committing immediately:

```bash
logi-led set-all --color ff0000 --no-commit
```

Display available key names and effect descriptions:

```bash
logi-led help-keys
logi-led help-effects
```

## License

Licensed under the terms of the GNU General Public License v3.0. See [`LICENSE`](LICENSE) for details.
