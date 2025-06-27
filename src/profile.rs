use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, StdinLock},
    path::Path,
};

use anyhow::Result;

use crate::keyboard::parser::{
    parse_board_mode, parse_color, parse_key, parse_key_group, parse_native_effect,
    parse_native_effect_part, parse_native_effect_storage, parse_period, parse_startup_mode,
    parse_u8,
};
use crate::keyboard::{Color, KeyValue, NativeEffect, NativeEffectStorage, api::KeyboardApi};

/// Parse a profile from any buffered reader
pub fn parse_profile<K>(kbd: &mut K, mut reader: impl BufRead) -> Result<()>
where
    K: KeyboardApi,
{
    let mut vars = HashMap::<String, String>::new();
    let mut keys = Vec::<KeyValue>::new();
    let mut line = String::new();

    while reader.read_line(&mut line)? != 0 {
        // Strip trailing newline(s) and comments
        if let Some(idx) = line.find('#') {
            line.truncate(idx);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        // Tokenize with cheap borrowing where possible
        let mut args: Vec<Cow<'_, str>> = Vec::with_capacity(8);
        for tok in trimmed.split_whitespace() {
            let cow = if let Some(var) = tok.strip_prefix('$') {
                vars.get(var)
                    .map(|v| Cow::Owned(v.clone()))
                    .unwrap_or_else(|| Cow::Borrowed(tok))
            } else {
                Cow::Borrowed(tok)
            };
            args.push(cow);
        }

        match args.first().map(Cow::as_ref) {
            Some("var") if args.len() >= 3 => {
                vars.insert(args[1].to_string(), args[2].to_string());
            }

            Some("c") => {
                if !keys.is_empty() {
                    kbd.set_keys(&keys)?;
                    keys.clear();
                }
                kbd.commit()?;
            }

            Some("a") => {
                if let Some(color) = parse_color(&args[1]) {
                    kbd.set_all_keys(color)?;
                }
            }

            Some("g") if args.len() >= 3 => {
                if let (Some(group), Some(color)) =
                    (parse_key_group(&args[1]), parse_color(&args[2]))
                {
                    kbd.set_group_keys(group, color)?;
                }
            }

            Some("k") if args.len() >= 3 => {
                if let (Some(key), Some(color)) = (parse_key(&args[1]), parse_color(&args[2])) {
                    keys.push(KeyValue { key, color });
                }
            }

            Some("r") if args.len() >= 3 => {
                if let (Some(region), Some(color)) = (parse_u8(&args[1]), parse_color(&args[2])) {
                    kbd.set_region(region, color)?;
                }
            }

            Some("mr") => {
                if let Some(v) = parse_u8(&args[1]) {
                    kbd.set_mr_key(v)?;
                }
            }

            Some("mn") => {
                if let Some(v) = parse_u8(&args[1]) {
                    kbd.set_mn_key(v)?;
                }
            }

            Some("gkm") => {
                if let Some(v) = parse_u8(&args[1]) {
                    kbd.set_gkeys_mode(v)?;
                }
            }

            Some("sm") => {
                if let Some(mode) = parse_startup_mode(&args[1]) {
                    kbd.set_startup_mode(mode)?;
                }
            }

            Some("obm") => {
                if let Some(mode) = parse_board_mode(&args[1]) {
                    kbd.set_on_board_mode(mode)?;
                }
            }

            Some("fx") if args.len() >= 3 => {
                // fx <effect> <part> [color|period] [...]
                if let (Some(effect), Some(part)) = (
                    parse_native_effect(&args[1]),
                    parse_native_effect_part(&args[2]),
                ) {
                    let mut period: Option<std::time::Duration> = None;
                    let color: Option<Color>;
                    let mut storage = NativeEffectStorage::None;

                    match effect {
                        NativeEffect::Color => {
                            color = args.get(3).and_then(|arg| parse_color(arg));
                        }
                        NativeEffect::Breathing => {
                            color = args.get(3).and_then(|arg| parse_color(arg));
                            period = args.get(4).and_then(|arg| parse_period(arg));
                        }
                        _ => {
                            period = args.get(3).and_then(|arg| parse_period(arg));
                            color = args.get(4).and_then(|arg| parse_color(arg));
                        }
                    }

                    if let Some(s) = args.get(5).and_then(|arg| parse_native_effect_storage(arg)) {
                        storage = s;
                    }

                    kbd.set_fx(
                        effect,
                        part,
                        period.unwrap_or_default(),
                        color.unwrap_or_default(),
                        storage,
                    )?;
                }
            }

            _ => {} // Unknown or malformed command - silently skip
        }

        line.clear(); // reuse the same buffer
    }

    if !keys.is_empty() {
        kbd.set_keys(&keys)?;
    }

    Ok(())
}

/// Load a profile from a file path.
pub fn load_profile<K>(kbd: &mut K, path: impl AsRef<Path>) -> Result<()>
where
    K: KeyboardApi,
{
    let file = File::open(path)?;
    parse_profile(kbd, BufReader::new(file))
}

/// Parse a profile from standard input.
pub fn load_profile_stdin<K>(kbd: &mut K, stdin: StdinLock<'_>) -> Result<()>
where
    K: KeyboardApi,
{
    parse_profile(kbd, stdin)
}
