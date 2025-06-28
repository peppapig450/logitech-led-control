use core::fmt;
use core::str::FromStr;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use super::parser::{parse_color, parse_key};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
#[strum(ascii_case_insensitive, serialize_all = "kebab-case")]
pub enum StartupMode {
    /// built-in wave effect on startup
    Wave = 0x01,
    /// Solid color set by firmware
    Color,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
#[strum(ascii_case_insensitive, serialize_all = "kebab-case")]
pub enum OnBoardMode {
    Board = 0x01,
    Software,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
#[strum(ascii_case_insensitive, serialize_all = "kebab-case")]
pub enum KeyGroup {
    Logo = 0x00,
    Indicators,
    Multimedia,
    GKeys,
    FKeys,
    Modifiers,
    Functions,
    Arrows,
    Numeric,
    Keys,
}

impl FromStr for KeyGroup {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_lowercase().replace('_', "-");
        match normalized.as_str() {
            "logo" => Ok(KeyGroup::Logo),
            "indicators" => Ok(KeyGroup::Indicators),
            "multimedia" => Ok(KeyGroup::Multimedia),
            "gkeys" | "g-keys" => Ok(KeyGroup::GKeys),
            "fkeys" | "f-keys" => Ok(KeyGroup::FKeys),
            "modifiers" => Ok(KeyGroup::Modifiers),
            "functions" => Ok(KeyGroup::Functions),
            "arrows" => Ok(KeyGroup::Arrows),
            "numeric" => Ok(KeyGroup::Numeric),
            "keys" => Ok(KeyGroup::Keys),
            _ => Err(format!("invalid key group: {s}")),
        }
    }
}

impl KeyGroup {
    /// Lazily iterate the keys that belong to this group.
    pub fn keys(self) -> impl Iterator<Item = Key> {
        Key::iter().filter(move |k| k.group() == self as u8)
    }
}

/// Two-byte scan code: high byte = address group, low byte = HID/key code.
///
/// *We keep every discriminant explicit so the layout never changes.*
#[repr(u16)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumIter,
    IntoPrimitive,    // `into(): u16`
    TryFromPrimitive, // `Key::try_from(u16)`
)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
pub enum Key {
    Logo = 0x0001,
    Logo2 = 0x0002,
    Backlight = 0x0101,
    Game = 0x0102,
    Caps = 0x0103,
    Scroll = 0x0104,
    Num = 0x0105,
    Next = 0x02b5,
    Prev = 0x02b6,
    Stop = 0x02b7,
    Play = 0x02cd,
    Mute = 0x02e2,
    G1 = 0x0301,
    G2 = 0x0302,
    G3 = 0x0303,
    G4 = 0x0304,
    G5 = 0x0305,
    G6 = 0x0306,
    G7 = 0x0307,
    G8 = 0x0308,
    G9 = 0x0309,
    A = 0x0404,
    B = 0x0405,
    C = 0x0406,
    D = 0x0407,
    E = 0x0408,
    F = 0x0409,
    G = 0x040a,
    H = 0x040b,
    I = 0x040c,
    J = 0x040d,
    K = 0x040e,
    L = 0x040f,
    M = 0x0410,
    N = 0x0411,
    O = 0x0412,
    P = 0x0413,
    Q = 0x0414,
    R = 0x0415,
    S = 0x0416,
    T = 0x0417,
    U = 0x0418,
    V = 0x0419,
    W = 0x041a,
    X = 0x041b,
    Y = 0x041c,
    Z = 0x041d,
    N1 = 0x041e,
    N2 = 0x041f,
    N3 = 0x0420,
    N4 = 0x0421,
    N5 = 0x0422,
    N6 = 0x0423,
    N7 = 0x0424,
    N8 = 0x0425,
    N9 = 0x0426,
    N0 = 0x0427,
    Enter = 0x0428,
    Esc = 0x0429,
    Backspace = 0x042a,
    Tab = 0x042b,
    Space = 0x042c,
    Minus = 0x042d,
    Equal = 0x042e,
    OpenBracket = 0x042f,
    CloseBracket = 0x0430,
    Backslash = 0x0431,
    Dollar = 0x0432,
    Semicolon = 0x0433,
    Quote = 0x0434,
    Tilde = 0x0435,
    Comma = 0x0436,
    Period = 0x0437,
    Slash = 0x0438,
    CapsLock = 0x0439,
    F1 = 0x043a,
    F2 = 0x043b,
    F3 = 0x043c,
    F4 = 0x043d,
    F5 = 0x043e,
    F6 = 0x043f,
    F7 = 0x0440,
    F8 = 0x0441,
    F9 = 0x0442,
    F10 = 0x0443,
    F11 = 0x0444,
    F12 = 0x0445,
    PrintScreen = 0x0446,
    ScrollLock = 0x0447,
    PauseBreak = 0x0448,
    Insert = 0x0449,
    Home = 0x044a,
    PageUp = 0x044b,
    Del = 0x044c,
    End = 0x044d,
    PageDown = 0x044e,
    ArrowRight = 0x044f,
    ArrowLeft = 0x0450,
    ArrowBottom = 0x0451,
    ArrowTop = 0x0452,
    NumLock = 0x0453,
    NumSlash = 0x0454,
    NumAsterisk = 0x0455,
    NumMinus = 0x0456,
    NumPlus = 0x0457,
    NumEnter = 0x0458,
    Num1 = 0x0459,
    Num2 = 0x045a,
    Num3 = 0x045b,
    Num4 = 0x045c,
    Num5 = 0x045d,
    Num6 = 0x045e,
    Num7 = 0x045f,
    Num8 = 0x0460,
    Num9 = 0x0461,
    Num0 = 0x0462,
    NumDot = 0x0463,
    IntlBackslash = 0x0464,
    Menu = 0x0465,
    AbntSlash = 0x0487,
    CtrlLeft = 0x04e0,
    ShiftLeft = 0x04e1,
    AltLeft = 0x04e2,
    WinLeft = 0x04e3,
    CtrlRight = 0x04e4,
    ShiftRight = 0x04e5,
    AltRight = 0x04e6,
    WinRight = 0x04e7,
}

impl Key {
    /// Address-group nibble (0 ═ logo, 1 ═ indicators, …).
    #[inline]
    pub const fn group(self) -> u8 {
        (self as u16 >> 8) as u8
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?} (0x{:04x})", *self as u16)
    }
}

impl FromStr for Key {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_key(s).ok_or_else(|| format!("invalid key: {s}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Default for Color {
    fn default() -> Self {
        Color {
            red: 255,
            green: 255,
            blue: 255,
        } // white
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_color(s).ok_or_else(|| format!("invalid color: {s}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyValue {
    pub key: Key,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
    pub model: super::KeyboardModel,
}
