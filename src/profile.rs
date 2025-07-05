use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, StdinLock},
    path::Path,
};
use serde::Deserialize;

use anyhow::{Result, anyhow};

use crate::keyboard::parser::{
    parse_board_mode, parse_color, parse_key, parse_key_group, parse_native_effect,
    parse_native_effect_part, parse_native_effect_storage, parse_period, parse_startup_mode,
    parse_u8,
};
use crate::keyboard::{Color, KeyValue, NativeEffect, NativeEffectStorage, api::KeyboardApi};

#[derive(Deserialize)]
struct Profile {
    all: Option<String>,
    #[serde(default)]
    groups: Vec<GroupEntry>,
    #[serde(default)]
    key: Vec<KeyEntry>,
    #[serde(default)]
    regions: Vec<RegionEntry>,
    #[serde(default)]
    effects: Vec<EffectEntry>,
    mr: Option<u8>,
    mn: Option<u8>,
    gkeys_mode: Option<u8>,
    startup_mode: Option<String>,
    on_board_mode: Option<String>,
}

#[derive(Deserialize)]
struct GroupEntry {
    group: String,
    color: String,
}

#[derive(Deserialize)]
struct KeyEntry {
    key: String,
    color: String,
}

#[derive(Deserialize)]
struct RegionEntry {
    region: String,
    color: String,
}

#[derive(Deserialize)]
struct EffectEntry {
    effect: String,
    part: String,
    #[serde(default)]
    period: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    storage: Option<String>,
}

/// Parse a profile from any buffered reader
pub fn parse_profile<K>(kbd: &mut K, mut reader: impl BufRead, strict: bool) -> Result<()>
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
                    .map_or_else(|| Cow::Borrowed(tok), |v| Cow::Owned(v.clone()))
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

            _ => {
                if strict {
                    return Err(anyhow!("unknown command: {trimmed}"));
                }
                eprintln!("warning: unknown command: {trimmed}");
            }
        }

        line.clear(); // reuse the same buffer
    }

    if !keys.is_empty() {
        kbd.set_keys(&keys)?;
    }

    Ok(())
}

/// Load a profile from a file path.
pub fn load_profile<K>(kbd: &mut K, path: impl AsRef<Path>, strict: bool) -> Result<()>
where
    K: KeyboardApi,
{
    let file = File::open(path)?;
    parse_profile(kbd, BufReader::new(file), strict)
}

/// Parse a profile from standard input.
pub fn load_profile_stdin<K>(kbd: &mut K, stdin: StdinLock<'_>, strict: bool) -> Result<()>
where
    K: KeyboardApi,
{
    parse_profile(kbd, stdin, strict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyboard::{
        Color, Key, KeyGroup, KeyValue, NativeEffect, NativeEffectPart, NativeEffectStorage,
        api::KeyboardApi,
    };
    use std::time::Duration;

    #[derive(Default)]
    struct MockKeyboard {
        commits: usize,
        all_calls: Vec<Color>,
        group_calls: Vec<(KeyGroup, Color)>,
        key_calls: Vec<Vec<KeyValue>>, // each call collects slice
        region_calls: Vec<(u8, Color)>,
        fx_calls: Vec<(
            NativeEffect,
            NativeEffectPart,
            Duration,
            Color,
            NativeEffectStorage,
        )>,
    }

    impl KeyboardApi for MockKeyboard {
        fn commit(&mut self) -> anyhow::Result<()> {
            self.commits += 1;
            Ok(())
        }

        fn set_all_keys(&mut self, color: Color) -> anyhow::Result<()> {
            self.all_calls.push(color);
            Ok(())
        }

        fn set_group_keys(&mut self, group: KeyGroup, color: Color) -> anyhow::Result<()> {
            self.group_calls.push((group, color));
            Ok(())
        }

        fn set_keys(&mut self, keys: &[KeyValue]) -> anyhow::Result<()> {
            self.key_calls.push(keys.to_vec());
            Ok(())
        }

        fn set_region(&mut self, region: u8, color: Color) -> anyhow::Result<()> {
            self.region_calls.push((region, color));
            Ok(())
        }

        fn set_fx(
            &mut self,
            effect: NativeEffect,
            part: NativeEffectPart,
            period: Duration,
            color: Color,
            storage: NativeEffectStorage,
        ) -> anyhow::Result<()> {
            self.fx_calls.push((effect, part, period, color, storage));
            Ok(())
        }
    }

    #[test]
    fn parse_keys_and_commit() {
        let input = "k a ff0000\nk b 00ff00\nc\n";
        let mut mock = MockKeyboard::default();
        parse_profile(&mut mock, input.as_bytes(), true).unwrap();

        assert_eq!(mock.key_calls.len(), 1);
        assert_eq!(
            mock.key_calls[0],
            vec![
                KeyValue {
                    key: Key::A,
                    color: Color {
                        red: 0xff,
                        green: 0x00,
                        blue: 0x00
                    }
                },
                KeyValue {
                    key: Key::B,
                    color: Color {
                        red: 0x00,
                        green: 0xff,
                        blue: 0x00
                    }
                },
            ]
        );
        assert_eq!(mock.commits, 1);
    }

    #[test]
    fn parse_group_region_effect() {
        let input = "a 010203\ng arrows ff0000\nr 2 00ff00\nfx color keys ff0000\n";
        let mut mock = MockKeyboard::default();
        parse_profile(&mut mock, input.as_bytes(), true).unwrap();

        assert_eq!(
            mock.all_calls,
            vec![Color {
                red: 1,
                green: 2,
                blue: 3
            }]
        );
        assert_eq!(
            mock.group_calls,
            vec![(
                KeyGroup::Arrows,
                Color {
                    red: 0xff,
                    green: 0x00,
                    blue: 0x00
                }
            )]
        );
        assert_eq!(
            mock.region_calls,
            vec![(
                2,
                Color {
                    red: 0x00,
                    green: 0xff,
                    blue: 0x00
                }
            )]
        );
        assert_eq!(mock.fx_calls.len(), 1);
        let (eff, part, period, color, storage) = &mock.fx_calls[0];
        assert_eq!(*eff, NativeEffect::Color);
        assert_eq!(*part, NativeEffectPart::Keys);
        assert_eq!(*period, Duration::from_millis(0));
        assert_eq!(
            *color,
            Color {
                red: 0xff,
                green: 0x00,
                blue: 0x00
            }
        );
        assert_eq!(*storage, NativeEffectStorage::None);
    }

    #[test]
    fn unknown_command_non_strict() {
        let input = "foo\n";
        let mut mock = MockKeyboard::default();
        parse_profile(&mut mock, input.as_bytes(), false).unwrap();
        assert!(mock.commits == 0);
        assert!(mock.key_calls.is_empty());
    }

    #[test]
    fn unknown_command_strict() {
        let input = "bar\n";
        let mut mock = MockKeyboard::default();
        let err = parse_profile(&mut mock, input.as_bytes(), true).unwrap_err();
        assert!(err.to_string().contains("unknown command"));
    }
}
