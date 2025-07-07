use super::Color;
use phf::{Map, phf_map};

/// Mapping of common color names to RGB values.
pub static COLOR_LOOKUP: Map<&'static str, Color> = phf_map! {
    "black"   => Color::new(0x00, 0x00, 0x00),
    "white"   => Color::new(0xff, 0xff, 0xff),
    "red"     => Color::new(0xff, 0x00, 0x00),
    "green"   => Color::new(0x00, 0xff, 0x00),
    "blue"    => Color::new(0x00, 0x00, 0xff),
    "yellow"  => Color::new(0xff, 0xff, 0x00),
    "cyan"    => Color::new(0x00, 0xff, 0xff),
    "magenta" => Color::new(0xff, 0x00, 0xff),
    "orange"  => Color::new(0xff, 0xa5, 0x00),
    "purple"  => Color::new(0x80, 0x00, 0x80),
    "pink"    => Color::new(0xff, 0xc0, 0xcb),
};

/// Help text listing all supported color names.
pub const COLOR_HELP: &str = concat!(
    "Color value as rrggbb, rr, or name (",
    "black, white, red, green, blue, yellow, cyan, magenta, orange, purple, pink",
    ")",
);

/// Iterate all known color names.
pub fn color_names() -> impl Iterator<Item = &'static str> {
    COLOR_LOOKUP.keys().copied()
}

/// Look up a color name, ignoring ASCII case.
pub fn lookup_color(name: &str) -> Option<Color> {
    let lower = name.to_ascii_lowercase();
    COLOR_LOOKUP.get(lower.as_str()).copied()
}
