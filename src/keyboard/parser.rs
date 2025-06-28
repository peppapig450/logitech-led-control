use core::time::Duration;
use phf::{Map, phf_map};
use std::borrow::Cow;

use super::{
    Color, Key, KeyGroup, OnBoardMode, StartupMode,
    effects::{NativeEffect, NativeEffectPart, NativeEffectStorage},
};

/// Parse a startup mode string.
pub fn parse_startup_mode(s: &str) -> Option<StartupMode> {
    s.parse::<StartupMode>().ok()
}

/// Parse an on-board mode string.
pub fn parse_board_mode(s: &str) -> Option<OnBoardMode> {
    s.parse::<OnBoardMode>().ok()
}

/// Cheap ASCII lowercase without always heap-allocating
#[inline]
fn ascii_lower<'a>(input: &'a str) -> Cow<'a, str> {
    if input.bytes().all(|byte| !byte.is_ascii_uppercase()) {
        // Already lowercase - avoid allocation
        Cow::Borrowed(input)
    } else {
        // Convert and allocate only when needed
        Cow::Owned(input.to_ascii_lowercase())
    }
}

/// Parse a color in hexadecimal `rrggbb` form (optionally `rr` for G610).
pub fn parse_color(val: &str) -> Option<Color> {
    // Accept "rrggbb" or "rr" (G610 grayscale). Optional leading '#'.
    let v = val.trim_start_matches('#');

    let bytes: [u8; 3] = match v.len() {
        6 => {
            let r = u8::from_str_radix(&v[0..2], 16).ok()?;
            let g = u8::from_str_radix(&v[2..4], 16).ok()?;
            let b = u8::from_str_radix(&v[4..6], 16).ok()?;
            [r, g, b]
        }
        2 => {
            let byte = u8::from_str_radix(v, 16).ok()?;
            [byte, byte, byte] // grey ramp: rr -> rr rr rr
        }
        _ => return None,
    };

    Some(Color {
        red: bytes[0],
        green: bytes[1],
        blue: bytes[2],
    })
}

/// Parse a key group name.
pub fn parse_key_group(s: &str) -> Option<KeyGroup> {
    s.parse::<KeyGroup>().ok()
}

/// All canonical and alias spellings -> `Key`.
/// Keep the *left-hand-side* lowercase so we can just lowercase the input.
static KEY_LOOKUP: Map<&'static str, Key> = phf_map! {
    // logo / indicators
    "logo" => Key::Logo,
    "logo2" => Key::Logo2,
    "backlight" => Key::Backlight,
    "back_light" => Key::Backlight,
    "light" => Key::Backlight,
    "game" => Key::Game,
    "gamemode" => Key::Game,
    "game_mode" => Key::Game,
    "caps" => Key::Caps,
    "capsindicator" => Key::Caps,
    "caps_indicator" => Key::Caps,
    "scroll" => Key::Scroll,
    "scrollindicator" => Key::Scroll,
    "scroll_indicator" => Key::Scroll,
    "num" => Key::Num,
    "numindicator" => Key::Num,
    "num_indicator" => Key::Num,

    // multimedia
    "next" => Key::Next,
    "prev" => Key::Prev,
    "previous" => Key::Prev,
    "stop" => Key::Stop,
    "play" => Key::Play,
    "playpause" => Key::Play,
    "play_pause" => Key::Play,
    "mute" => Key::Mute,

    // function keys
    "f1"  => Key::F1,   "f2"  => Key::F2,   "f3"  => Key::F3,
    "f4"  => Key::F4,   "f5"  => Key::F5,   "f6"  => Key::F6,
    "f7"  => Key::F7,   "f8"  => Key::F8,   "f9"  => Key::F9,
    "f10" => Key::F10,  "f11" => Key::F11,  "f12" => Key::F12,

    //  arrow / navigation / numpad
    "arrowright" => Key::ArrowRight,
    "right"      => Key::ArrowRight,
    "arrowleft"  => Key::ArrowLeft,
    "left"       => Key::ArrowLeft,
    "arrowtop"   => Key::ArrowTop,
    "up"         => Key::ArrowTop,
    "arrowbottom"=> Key::ArrowBottom,
    "down"       => Key::ArrowBottom,

    "insert"     => Key::Insert,
    "home"       => Key::Home,
    "pageup"     => Key::PageUp,
    "page_up"    => Key::PageUp,
    "pgup"       => Key::PageUp,
    "delete"     => Key::Del,
    "del"        => Key::Del,
    "end"        => Key::End,
    "pagedown"   => Key::PageDown,
    "page_down"  => Key::PageDown,
    "pgdn" => Key::PageDown,

    "numlock"     => Key::NumLock,
    "num_lock"    => Key::NumLock,
    "num_lock_key" => Key::NumLock,
    "num/"        => Key::NumSlash,
    "numslash"      => Key::NumSlash,
    "num_slash" => Key::NumSlash,
    "numasterisk" => Key::NumAsterisk,
    "num*"        => Key::NumAsterisk,
    "num-"        => Key::NumMinus,
    "numplus"     => Key::NumPlus,
    "num+"        => Key::NumPlus,
    "numenter"    => Key::NumEnter,
    "num_enter"   => Key::NumEnter,
    "num1"        => Key::Num1,
    "num2"        => Key::Num2,
    "num3"        => Key::Num3,
    "num4"        => Key::Num4,
    "num5"        => Key::Num5,
    "num6"        => Key::Num6,
    "num7"        => Key::Num7,
    "num8"        => Key::Num8,
    "num9"        => Key::Num9,
    "num0"        => Key::Num0,
    "numdot"      => Key::NumDot,
    "num."        => Key::NumDot,

    //  g-keys
    "g1" => Key::G1, "g2" => Key::G2, "g3" => Key::G3,
    "g4" => Key::G4, "g5" => Key::G5, "g6" => Key::G6,
    "g7" => Key::G7, "g8" => Key::G8, "g9" => Key::G9,

    // printable symbols & letters
    "~"  => Key::Tilde,      "-" => Key::Minus,      "=" => Key::Equal,
    "["  => Key::OpenBracket, "]" => Key::CloseBracket,
    "\\" => Key::Backslash,  ";" => Key::Semicolon,
    "\"" => Key::Quote,     "$" => Key::Dollar,
    ","  => Key::Comma,      "." => Key::Period,
    "/"  => Key::Slash,

    "enter"     => Key::Enter,
    "return"    => Key::Enter,
    "enter_key" => Key::Enter,
    "esc"       => Key::Esc,
    "escape"    => Key::Esc,
    "escape_key" => Key::Esc,
    "backspace" => Key::Backspace,
    "tab"       => Key::Tab,
    "space"     => Key::Space,
    "capslock"  => Key::CapsLock,
    "caps_lock" => Key::CapsLock,
    "printscreen" => Key::PrintScreen,
    "print"        => Key::PrintScreen,
    "print_screen" => Key::PrintScreen,
    "scroll_lock" => Key::ScrollLock,
    "scrolllock"  => Key::ScrollLock,
    "pause"       => Key::PauseBreak,
    "pause_break" => Key::PauseBreak,

    "intlbackslash" => Key::IntlBackslash,
    "menu"         => Key::Menu,
    "abntslash"    => Key::AbntSlash,

    "ctrlleft"   => Key::CtrlLeft,
    "lctrl"      => Key::CtrlLeft,
    "leftctrl"   => Key::CtrlLeft,
    "controlleft"=> Key::CtrlLeft,
    "shiftleft"  => Key::ShiftLeft,
    "lshift"     => Key::ShiftLeft,
    "leftshift"  => Key::ShiftLeft,
    "altleft"    => Key::AltLeft,
    "lalt"       => Key::AltLeft,
    "leftalt"    => Key::AltLeft,
    "winleft"    => Key::WinLeft,
    "lwin"       => Key::WinLeft,
    "ctrlright"  => Key::CtrlRight,
    "rctrl"      => Key::CtrlRight,
    "rightctrl"  => Key::CtrlRight,
    "controlright"=> Key::CtrlRight,
    "shiftright" => Key::ShiftRight,
    "rshift"     => Key::ShiftRight,
    "rightshift" => Key::ShiftRight,
    "altright"   => Key::AltRight,
    "ralt"       => Key::AltRight,
    "rightalt"   => Key::AltRight,
    "winright"   => Key::WinRight,
    "rwin"       => Key::WinRight,

    // alphanumeric keys
    "a" => Key::A, "b" => Key::B, "c" => Key::C, "d" => Key::D,
    "e" => Key::E, "f" => Key::F, "g" => Key::G, "h" => Key::H,
    "i" => Key::I, "j" => Key::J, "k" => Key::K, "l" => Key::L,
    "m" => Key::M, "n" => Key::N, "o" => Key::O, "p" => Key::P,
    "q" => Key::Q, "r" => Key::R, "s" => Key::S, "t" => Key::T,
    "u" => Key::U, "v" => Key::V, "w" => Key::W, "x" => Key::X,
    "y" => Key::Y, "z" => Key::Z,

    "1" => Key::N1, "2" => Key::N2, "3" => Key::N3,
    "4" => Key::N4, "5" => Key::N5, "6" => Key::N6,
    "7" => Key::N7, "8" => Key::N8, "9" => Key::N9,
    "0" => Key::N0,
};

/// Case-insensitive parse of a key name / alias.
/// *Single ASCII letter or digit* is handled in O(1) without the map.
pub fn parse_key(s: &str) -> Option<Key> {
    let lower_cow = ascii_lower(s);
    let lower = lower_cow.as_ref();

    if let Some(key) = KEY_LOOKUP.get(lower) {
        return Some(*key);
    }

    // single-character fallback; a-z, 0-9
    if lower.len() == 1 {
        return Some(match lower.as_bytes()[0] {
            b'a'..=b'z' => Key::try_from((lower.as_bytes()[0] - b'a') as u16).ok()?, // uses repr order
            b'0' => Key::N0,
            b'1' => Key::N1,
            b'2' => Key::N2,
            b'3' => Key::N3,
            b'4' => Key::N4,
            b'5' => Key::N5,
            b'6' => Key::N6,
            b'7' => Key::N7,
            b'8' => Key::N8,
            b'9' => Key::N9,
            _ => return None,
        });
    }
    None
}

pub fn parse_period(val: &str) -> Option<Duration> {
    // human-friendly: "200ms", "2s", or hex byte ("ff") x 256 ms
    let v = val.trim();

    // 1. explicit seconds / milliseconds
    if let Some(stripped) = v.strip_suffix(|c: char| c.eq_ignore_ascii_case(&'s')) {
        if let Some(ms) = stripped.strip_suffix(|c: char| c.eq_ignore_ascii_case(&'m')) {
            return ms.parse::<u64>().ok().map(Duration::from_millis);
        }
        return stripped.parse::<u64>().ok().map(Duration::from_secs);
    }

    // 2. hex byte (length 1 or 2)
    let hex: Cow<'_, str> = if v.len() == 1 {
        // avoid allocation; build two-char stack buffer
        let mut buf = [0u8; 2];
        buf[0] = b'0';
        buf[1] = v.as_bytes()[0];
        Cow::Owned(core::str::from_utf8(&buf).unwrap().to_string())
    } else {
        Cow::Borrowed(v)
    };

    if hex.len() == 2 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        let byte = u8::from_str_radix(&hex, 16).ok()?;
        return Some(Duration::from_millis(u64::from(byte) << 8));
    }

    None
}

/// Parse a native effect name.
pub fn parse_native_effect(s: &str) -> Option<NativeEffect> {
    s.parse::<NativeEffect>().ok()
}

/// Parse a native effect part string.
pub fn parse_native_effect_part(s: &str) -> Option<NativeEffectPart> {
    s.parse::<NativeEffectPart>().ok()
}

/// Parse a native effect storage string.
pub fn parse_native_effect_storage(s: &str) -> Option<NativeEffectStorage> {
    s.parse::<NativeEffectStorage>().ok()
}

/// Parse a u8 value from decimal or hexadecimal form.
pub fn parse_u8(val: &str) -> Option<u8> {
    u8::from_str_radix(val, 16)
        .ok()
        .or_else(|| val.parse::<u8>().ok())
}

/// Parse a u16 value from decimal or hexadecimal form
pub fn parse_u16(val: &str) -> Option<u16> {
    u16::from_str_radix(val, 16)
        .ok()
        .or_else(|| val.parse::<u16>().ok())
}
