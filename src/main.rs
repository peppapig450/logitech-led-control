use clap::{Args, Parser, Subcommand, ValueHint};
use std::path::PathBuf;

use keyboard::api::KeyboardApi;

mod commands;
mod keyboard;
mod profile;

use crate::keyboard::{
    Color, Key, KeyGroup, NativeEffect, NativeEffectPart, NativeEffectStorage, OnBoardMode,
    StartupMode,
    device::Keyboard,
    parser::{parse_period, parse_u8, parse_u16},
};
use crate::{
    commands::{list_keyboards, print_device},
    keyboard::{
        KeyboardModel,
        model::{self, LOGITECH_VENDOR_ID},
    },
};

// Control LEDS via HID
#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Device vendor ID (hex or decimal)
    #[arg(long = "dv", value_parser = parse_u16_arg)]
    vendor_id: Option<u16>,

    /// Device product ID (hex or decimal)
    #[arg(long = "dp", value_parser = parse_u16_arg)]
    product_id: Option<u16>,

    /// Test unsupported keyboard with a specific protocol (1-4)
    #[arg(long = "tuk", value_parser = parse_u8_arg)]
    protocol: Option<u8>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Clone)]
struct SerialArg {
    #[arg(long)]
    serial: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all connected Logitech HID devices
    ListKeyboards,

    /// Open a specific keyboard and print its info
    PrintDevice {
        #[command(flatten)]
        serial: SerialArg,
    },

    /// Commit any buffered changes
    Commit {
        #[command(flatten)]
        serial: SerialArg,
    },

    /// Set all keys to a color
    SetAll {
        #[command(flatten)]
        serial: SerialArg,
        #[arg(short = 'a')]
        color: Color,
        #[arg(long)]
        no_commit: bool,
    },

    /// Set a key group color
    SetGroup {
        #[command(flatten)]
        serial: SerialArg,
        #[arg(short = 'g')]
        group: KeyGroup,
        color: Color,
        #[arg(long)]
        no_commit: bool,
    },

    /// Set an individual key color
    SetKey {
        #[command(flatten)]
        serial: SerialArg,
        #[arg(short = 'k')]
        key: Key,
        color: Color,
        #[arg(long)]
        no_commit: bool,
    },

    /// Set a region color
    SetRegion {
        #[command(flatten)]
        serial: SerialArg,
        /// Region index
        region: u8,
        color: Color,
    },

    /// Set the MR key value
    SetMr {
        #[command(flatten)]
        serial: SerialArg,
        value: u8,
    },

    /// Set the Mn key value
    SetMn {
        #[command(flatten)]
        serial: SerialArg,
        value: u8,
    },

    /// Set the G-keys mode
    GKeysMode {
        #[command(flatten)]
        serial: SerialArg,
        value: u8,
    },

    /// Load profile from a file
    LoadProfile {
        #[command(flatten)]
        serial: SerialArg,
        #[arg(value_hint = ValueHint::FilePath)]
        path: PathBuf,
    },

    /// Load profile from stdin
    PipeProfile {
        #[command(flatten)]
        serial: SerialArg,
    },

    /// Apply a lighting effect
    Fx {
        #[command(flatten)]
        serial: SerialArg,
        effect: NativeEffect,
        part: NativeEffectPart,
        #[arg(long, value_parser = parse_period_arg)]
        period: Option<std::time::Duration>,
        #[arg(long)]
        color: Option<Color>,
    },

    /// Store a lighting effect in memory
    FxStore {
        #[command(flatten)]
        serial: SerialArg,
        effect: NativeEffect,
        part: NativeEffectPart,
        #[arg(long, value_parser = parse_period_arg)]
        period: Option<std::time::Duration>,
        #[arg(long)]
        color: Option<Color>,
        storage: NativeEffectStorage,
    },

    /// Configure startup mode
    StartupMode {
        #[command(flatten)]
        serial: SerialArg,
        mode: StartupMode,
    },

    /// Configure on-board mode
    OnBoardMode {
        #[command(flatten)]
        serial: SerialArg,
        mode: OnBoardMode,
    },
}

impl Commands {
    fn run(&self, opts: &Cli) -> anyhow::Result<()> {
        match self {
            Commands::ListKeyboards => list_keyboards(),
            Commands::PrintDevice { serial } => print_device(serial.serial.as_deref()),
            Commands::Commit { serial } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                serial.serial.as_deref(),
                |kbd| kbd.commit(),
            ),
            Commands::SetAll {
                serial,
                color,
                no_commit,
            } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                serial.serial.as_deref(),
                |kbd| {
                    kbd.set_all_keys(*color)?;
                    if !no_commit {
                        kbd.commit()?;
                    }
                    Ok(())
                },
            ),
            Commands::SetGroup {
                serial,
                group,
                color,
                no_commit,
            } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                serial.serial.as_deref(),
                |kbd| {
                    kbd.set_group_keys(*group, *color)?;
                    if !no_commit {
                        kbd.commit()?;
                    }
                    Ok(())
                },
            ),
            Commands::SetKey {
                serial,
                key,
                color,
                no_commit,
            } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                serial.serial.as_deref(),
                |kbd| {
                    kbd.set_keys(&[keyboard::KeyValue {
                        key: *key,
                        color: *color,
                    }])?;
                    if !no_commit {
                        kbd.commit()?;
                    }
                    Ok(())
                },
            ),
            Commands::SetRegion {
                serial,
                region,
                color,
            } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                serial.serial.as_deref(),
                |kbd| {
                    kbd.set_region(*region, *color)?;
                    Ok(())
                },
            ),
            Commands::SetMr { serial, value } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                serial.serial.as_deref(),
                |kbd| kbd.set_mr_key(*value),
            ),
            Commands::SetMn { serial, value } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                serial.serial.as_deref(),
                |kbd| kbd.set_mn_key(*value),
            ),
            Commands::GKeysMode { serial, value } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                serial.serial.as_deref(),
                |kbd| kbd.set_gkeys_mode(*value),
            ),
            Commands::LoadProfile { serial, path } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                serial.serial.as_deref(),
                |kbd| profile::load_profile(kbd, &path),
            ),
            Commands::PipeProfile { serial } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                serial.serial.as_deref(),
                |kbd| {
                    let stdin = std::io::stdin();
                    profile::load_profile_stdin(kbd, stdin.lock())
                },
            ),
            Commands::Fx {
                serial,
                effect,
                part,
                period,
                color,
            } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                serial.serial.as_deref(),
                |kbd| {
                    kbd.set_fx(
                        *effect,
                        *part,
                        period.unwrap_or_default(),
                        color.unwrap_or_default(),
                        NativeEffectStorage::None,
                    )
                },
            ),
            Commands::FxStore {
                serial,
                effect,
                part,
                period,
                color,
                storage,
            } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                serial.serial.as_deref(),
                |kbd| {
                    kbd.set_fx(
                        *effect,
                        *part,
                        period.unwrap_or_default(),
                        color.unwrap_or_default(),
                        *storage,
                    )
                },
            ),
            Commands::StartupMode { serial, mode } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                serial.serial.as_deref(),
                |kbd| kbd.set_startup_mode(*mode),
            ),
            Commands::OnBoardMode { serial, mode } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                serial.serial.as_deref(),
                |kbd| kbd.set_on_board_mode(*mode),
            ),
        }
    }
}

fn parse_period_arg(s: &str) -> Result<std::time::Duration, String> {
    parse_period(s).ok_or_else(|| format!("invalid period: {s}"))
}

fn parse_u8_arg(s: &str) -> Result<u8, String> {
    parse_u8(s).ok_or_else(|| format!("Invalid u8 value: {s}"))
}

fn parse_u16_arg(s: &str) -> Result<u16, String> {
    parse_u16(s).ok_or_else(|| format!("Invalid u16 value: {s}"))
}

fn with_keyboard<F>(
    vendor_id: Option<u16>,
    product_id: Option<u16>,
    protocol: Option<u8>,
    serial: Option<&str>,
    mut f: F,
) -> anyhow::Result<()>
where
    F: FnMut(&mut Keyboard) -> anyhow::Result<()>,
{
    let vid = vendor_id.unwrap_or(LOGITECH_VENDOR_ID);
    let pid = product_id.unwrap_or(0);

    if let Some(model) = protocol.and_then(|id| match id {
        1 => Some(KeyboardModel::G810),
        2 => Some(KeyboardModel::G910),
        3 => Some(KeyboardModel::G213),
        4 => Some(KeyboardModel::G815),
        _ => None,
    }) {
        // NOTE: this could probably be a static sized array,
        // and the SUPPORTED_DEVICES and override could be a
        // triplet.
        model::set_supported_override(vec![(vid, pid, model)]);
    }

    let mut kbd = Keyboard::open(vid, pid, serial.as_deref())?;
    let res = f(&mut kbd);
    kbd.close().ok();
    model::clear_supported_override(); // NOTE: move this into `kbd.close`?
    res
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.command.run(&cli)
}
