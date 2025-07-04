use clap::{Args, CommandFactory, Parser, Subcommand, ValueHint};
use std::path::PathBuf;

use keyboard::api::KeyboardApi;

mod commands;
mod help;
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
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    propagate_version = true,
    arg_required_else_help = true
)]
struct Cli {
    /// Device vendor ID (hex or decimal)   [env: LOGI_VENDOR_ID=]
    #[arg(long = "vendor-id", short = 'v', value_parser = parse_u16_arg)]
    vendor_id: Option<u16>,

    /// Device product ID (hex or decimal)  [env: LOGI_PRODUCT_ID=]
    #[arg(long = "product-id", short = 'p', value_parser = parse_u16_arg)]
    product_id: Option<u16>,

    /// Test unsupported keyboard with a specific protocol (1-4)
    #[arg(long = "tuk", value_parser = parse_u8_arg)]
    protocol: Option<u8>,

    /// Fail on unknown commands in profiles
    #[arg(long, default_value_t = false, action)]
    strict: bool,

    /// Device serial number
    #[arg(long, global = true)]
    serial: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct ColorTarget {
    #[arg(short, long)]
    key: Option<Key>,
    #[arg(short, long)]
    group: Option<KeyGroup>,
    #[arg(short = 'A', long)]
    all: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all connected Logitech HID devices
    ListKeyboards,

    /// Open a specific keyboard and print its info
    PrintDevice,

    /// Commit any buffered changes
    Commit,

    /// Set key colors
    #[command(name = "set")]
    SetColor {
        #[command(flatten)]
        target: ColorTarget,
        #[arg(help = help::COLOR_HELP)]
        color: Color,
        #[arg(long)]
        no_commit: bool,
    },

    /// Set a region color
    SetRegion {
        /// Region index
        region: u8,
        #[arg(help = help::COLOR_HELP)]
        color: Color,
    },

    /// Set the MR key value
    SetMr { value: u8 },

    /// Set the Mn key value
    SetMn { value: u8 },

    /// Set the G-keys mode
    GKeysMode { value: u8 },

    /// Load profile from a file
    LoadProfile {
        #[arg(value_hint = ValueHint::FilePath)]
        path: PathBuf,
    },

    /// Load profile from stdin
    PipeProfile,

    /// Apply a lighting effect
    Fx {
        effect: NativeEffect,
        part: NativeEffectPart,
        #[arg(long, value_parser = parse_period_arg)]
        period: Option<std::time::Duration>,
        #[arg(long, help = help::COLOR_HELP)]
        color: Option<Color>,
    },

    /// Store a lighting effect in memory
    FxStore {
        effect: NativeEffect,
        part: NativeEffectPart,
        #[arg(long, value_parser = parse_period_arg)]
        period: Option<std::time::Duration>,
        #[arg(long, help = help::COLOR_HELP)]
        color: Option<Color>,
        storage: NativeEffectStorage,
    },

    /// Configure startup mode
    StartupMode { mode: StartupMode },

    /// Configure on-board mode
    OnBoardMode { mode: OnBoardMode },

    /// Display help for keys
    #[command(name = "help-keys")]
    HelpKeys,

    /// Display help for lighting effects
    #[command(name = "help-effects")]
    HelpEffects,

    /// Display help for color names
    #[command(name = "help-colors")]
    HelpColors,

    /// Show usage samples
    #[command(name = "help-samples")]
    HelpSamples,

    /// Generate shell completion scripts
    Completions { shell: clap_complete::Shell },
}

impl Commands {
    fn run(&self, opts: &Cli) -> anyhow::Result<()> {
        match self {
            Commands::ListKeyboards => list_keyboards(),
            Commands::PrintDevice => print_device(opts.serial.as_deref()),
            Commands::Commit => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                opts.serial.as_deref(),
                |kbd| kbd.commit(),
            ),
            Commands::SetColor {
                target,
                color,
                no_commit,
            } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                opts.serial.as_deref(),
                |kbd| {
                    if target.all {
                        kbd.set_all_keys(*color)?;
                    } else if let Some(group) = target.group {
                        kbd.set_group_keys(group, *color)?;
                    } else if let Some(key) = target.key {
                        kbd.set_keys(&[keyboard::KeyValue { key, color: *color }])?;
                    }
                    if !*no_commit {
                        kbd.commit()?;
                    }
                    Ok(())
                },
            ),
            Commands::SetRegion { region, color } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                opts.serial.as_deref(),
                |kbd| {
                    kbd.set_region(*region, *color)?;
                    Ok(())
                },
            ),
            Commands::SetMr { value } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                opts.serial.as_deref(),
                |kbd| kbd.set_mr_key(*value),
            ),
            Commands::SetMn { value } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                opts.serial.as_deref(),
                |kbd| kbd.set_mn_key(*value),
            ),
            Commands::GKeysMode { value } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                opts.serial.as_deref(),
                |kbd| kbd.set_gkeys_mode(*value),
            ),
            Commands::LoadProfile { path } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                opts.serial.as_deref(),
                |kbd| profile::load_profile(kbd, path, opts.strict),
            ),
            Commands::PipeProfile => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                opts.serial.as_deref(),
                |kbd| {
                    let stdin = std::io::stdin();
                    profile::load_profile_stdin(kbd, stdin.lock(), opts.strict)
                },
            ),
            Commands::Fx {
                effect,
                part,
                period,
                color,
            } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                opts.serial.as_deref(),
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
                effect,
                part,
                period,
                color,
                storage,
            } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                opts.serial.as_deref(),
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
            Commands::StartupMode { mode } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                opts.serial.as_deref(),
                |kbd| kbd.set_startup_mode(*mode),
            ),
            Commands::OnBoardMode { mode } => with_keyboard(
                opts.vendor_id,
                opts.product_id,
                opts.protocol,
                opts.serial.as_deref(),
                |kbd| kbd.set_on_board_mode(*mode),
            ),
            Commands::HelpKeys => {
                help::print_keys_help();
                Ok(())
            }
            &Commands::HelpEffects => {
                help::print_effects_help();
                Ok(())
            }
            &Commands::HelpColors => {
                help::print_colors_help();
                Ok(())
            }
            &Commands::HelpSamples => {
                help::print_samples_help();
                Ok(())
            }
            Commands::Completions { shell } => {
                let mut cmd = Cli::command();
                clap_complete::generate(*shell, &mut cmd, "logi-led", &mut std::io::stdout());
                Ok(())
            }
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

    let mut kbd = match Keyboard::open(vid, pid, serial) {
        Ok(k) => k,
        Err(e) => {
            model::clear_supported_override();
            return Err(e);
        }
    };
    f(&mut kbd)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.command.run(&cli)
}
