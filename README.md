# Logitech LED Control

A command-line utility written in Rust for controlling the lighting of supported Logitech keyboards. It allows listing attached devices, setting key colors or groups, applying effects, and loading profiles.

## Features

- Detect and open Logitech keyboards via HID or libusb backend.
- Set colors for individual keys, groups, or regions.
- Apply and store native lighting effects.
- Configure startup and onboard modes.
- Load lighting profiles from files or standard input.
- Drop-in compatible with profile files from [g810-led](g810-led/README.md).
- Supports structured TOML configuration files.

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

## Shell completions

To generate shell completion scripts, run:

```bash
logi-led completion bash > /etc/bash_completion.d/logi-led
```

## g810-led profiles

Existing profiles from the [g810-led](g810-led/README.md) project work
unchanged with `logi-led`. Use `load-profile` to load a profile from a file or
`pipe-profile` to read one from standard input. Sample profiles can be found in
the `g810-led/sample_profiles` directory.

## Structured profiles

Lighting setups can also be described with a structured TOML file.
Use `load-config` to apply one:

```bash
logi-led load-config myprofile.toml
```

Example configuration:

```toml
all = "010203"

[[groups]]
group = "arrows"
color = "ff0000"

[[key]]
key = "a"
color = "00ff00"

[[regions]]
region = "2"
color = "0000ff"

[[effects]]
effect = "color"
part = "keys"
color = "ff00ff"
```

## Acknowledgments

This project draws inspiration from [g810-led](https://github.com/MatMoul/g810-led), which pioneered command-line control of Logitech G-series keyboard lighting. While `logi-led` is an independent Rust implementation, we appreciate the groundwork laid by the g810-led project and its contributors.

## License

Licensed under the terms of the GNU General Public License v3.0. See [`LICENSE`](LICENSE) for details.