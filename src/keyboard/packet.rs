use crate::keyboard::{
    Color, Key, KeyValue, KeyboardModel, NativeEffect, NativeEffectPart, NativeEffectStorage,
};
use core::time::Duration;

/// Pad a packet to `size` bytes (20 or 64) with zeroes.
#[inline]
fn pad(mut data: Vec<u8>, size: usize) -> Vec<u8> {
    data.resize(size, 0x00);
    data
}

/// Constant, model-independent byte slices
type Packet = &'static [u8];

const PKT_COMMIT_GX: Packet = &[0x11, 0xff, 0x0c, 0x5a]; // G410/G512/…
const PKT_COMMIT_G815: Packet = &[0x11, 0xff, 0x10, 0x7f];
const PKT_COMMIT_G910: Packet = &[0x11, 0xff, 0x0f, 0x5d];

const PKT_ADDR_0: Packet = &[0x11, 0xff, 0x0c, 0x3a, 0x00, 0x10, 0x00, 0x01];
const PKT_ADDR_1: Packet = &[0x12, 0xff, 0x0c, 0x3a, 0x00, 0x40, 0x00, 0x05];
const PKT_ADDR_2: Packet = &[0x12, 0xff, 0x0c, 0x3a, 0x00, 0x02, 0x00, 0x05];
const PKT_ADDR_4C: Packet = &[0x12, 0xff, 0x0c, 0x3a, 0x00, 0x01, 0x00, 0x0e];
const PKT_ADDR_4F: Packet = &[0x12, 0xff, 0x0f, 0x3d, 0x00, 0x01, 0x00, 0x0e];
const PKT_ADDR_G910_0: Packet = &[0x11, 0xff, 0x0f, 0x3a, 0x00, 0x10, 0x00, 0x02];
const PKT_ADDR_G910_3: Packet = &[0x12, 0xff, 0x0f, 0x3e, 0x00, 0x04, 0x00, 0x09];
const PKT_ADDR_G815: Packet = &[0x11, 0xff, 0x10, 0x1c];

/// Packet used to commit changes to the device.
pub fn commit_packet(model: KeyboardModel) -> Option<Vec<u8>> {
    let slice = match model {
        KeyboardModel::G410
        | KeyboardModel::G512
        | KeyboardModel::G513
        | KeyboardModel::G610
        | KeyboardModel::G810
        | KeyboardModel::GPro => PKT_COMMIT_GX,
        KeyboardModel::G815 => PKT_COMMIT_G815,
        KeyboardModel::G910 => PKT_COMMIT_G910,
        // G213/G413 are non transactional
        _ => return None,
    };
    Some(pad(slice.to_vec(), 20))
}

/// Raw HID header for a key group.
fn group_address(model: KeyboardModel, group: u8) -> Option<Packet> {
    use KeyboardModel::*;

    match (model, group) {
        // Same mapping for these boards
        (G410 | G512 | G513 | GPro | G610 | G810, 0) => Some(PKT_ADDR_0),
        (G410 | G512 | G513 | GPro | G610 | G810 | G910, 1) => Some(PKT_ADDR_1),
        (G410 | G512 | G513 | GPro | G610 | G810, 4) => Some(PKT_ADDR_4C),
        (G610 | G810, 2) => Some(PKT_ADDR_2),

        (G815, _) => Some(PKT_ADDR_G815),

        (G910, 0) => Some(PKT_ADDR_G910_0),
        (G910, 3) => Some(PKT_ADDR_G910_3),
        (G910, 4) => Some(PKT_ADDR_4F),

        _ => None,
    }
}

/// Translate a [`Key`] into the byte identifier used by the G815.
fn g815_key_id(key: Key) -> Option<u8> {
    let low = key as u16 as u8;

    Some(match key {
        Key::Logo2
        | Key::Game
        | Key::Caps
        | Key::Scroll
        | Key::Num
        | Key::Stop
        | Key::G6
        | Key::G7
        | Key::G8
        | Key::G9 => return None,

        Key::Play => 0x9b,
        Key::Mute => 0x9c,
        Key::Next => 0x9d,
        Key::Prev => 0x9e,

        Key::CtrlLeft
        | Key::ShiftLeft
        | Key::AltLeft
        | Key::WinLeft
        | Key::CtrlRight
        | Key::ShiftRight
        | Key::AltRight
        | Key::WinRight => low.wrapping_sub(0x78),

        _ => match key.group() {
            0 => low.wrapping_add(0xd1),
            1 => low.wrapping_add(0x98),
            3 => low.wrapping_add(0xb3),
            4 => low.wrapping_sub(0x03),
            _ => return None,
        },
    })
}

/// Build a HID report that sets one or more keys.
/// The slice must contain keys from the same address group.
pub fn set_keys_packet(model: KeyboardModel, keys: &[KeyValue]) -> Option<Vec<u8>> {
    if keys.is_empty() {
        return None;
    }

    match model {
        KeyboardModel::G213 | KeyboardModel::G413 => None,

        KeyboardModel::G815 => {
            // G815 requires a single color for the entire packet
            let color = keys[0].color;
            if keys.iter().any(|k| k.color != color) {
                return None;
            }

            let mut data = Vec::with_capacity(20);
            data.extend_from_slice(&[0x11, 0xff, 0x10, 0x6c, color.red, color.green, color.blue]);

            for kv in keys.iter().take(13) {
                if let Some(id) = g815_key_id(kv.key) {
                    data.push(id);
                }
            }

            if data.len() < 20 {
                data.push(0xff); // sentinel
            }

            Some(pad(data, 20))
        }

        _ => {
            let group = keys[0].key.group();
            if keys.iter().any(|k| k.key.group() != group) {
                return None;
            }

            let size = if group == 0 { 20 } else { 64 };
            let max_keys = (size - 8) / 4;
            let mut data = group_address(model, group)?.to_vec();

            for kv in keys.iter().take(max_keys) {
                data.extend_from_slice(&[
                    kv.key as u16 as u8,
                    kv.color.red,
                    kv.color.green,
                    kv.color.blue,
                ]);
            }

            Some(pad(data, size))
        }
    }
}

/// Packet to set a region color (G213 only).
pub fn region_packet(model: KeyboardModel, region: u8, color: Color) -> Option<Vec<u8>> {
    if let KeyboardModel::G213 = model {
        Some(pad(
            vec![
                0x11,
                0xff,
                0x0c,
                0x3a,
                region,
                0x01,
                color.red,
                color.green,
                color.blue,
            ],
            20,
        ))
    } else {
        None
    }
}

/// Packet for built-in lighting effects.
pub fn native_effect_packet(
    model: KeyboardModel,
    effect: NativeEffect,
    part: NativeEffectPart,
    period: Duration,
    color: Color,
    storage: NativeEffectStorage,
) -> Option<Vec<u8>> {
    // The firmware uses part = 0xff to mean "all", which we don't support.
    if matches!(part, NativeEffectPart::All) {
        return None;
    }

    let (p0, p1) = match model {
        KeyboardModel::G213 | KeyboardModel::G413 => (0x0c, 0x3c),
        KeyboardModel::G410
        | KeyboardModel::G512
        | KeyboardModel::G513
        | KeyboardModel::G610
        | KeyboardModel::G810
        | KeyboardModel::GPro => (0x0d, 0x3c),
        KeyboardModel::G815 => (0x0f, 0x1c),
        KeyboardModel::G910 => (0x10, 0x3c),
        _ => return None,
    };

    let per_ms = period.as_millis() as u16;
    let effect_group = ((effect as u16) >> 8) as u8;

    let mut data = Vec::with_capacity(20);
    data.extend_from_slice(&[
        0x11,
        0xff,
        p0,
        p1,
        part as u8,
        effect_group,
        color.red,
        color.green,
        color.blue,
        (per_ms >> 8) as u8,
        (per_ms & 0xff) as u8,
        (per_ms >> 8) as u8,
        (per_ms & 0xff) as u8,
        (effect as u16 & 0xff) as u8,
        0x64,
        (per_ms >> 8) as u8,
        storage as u8,
        0x00,
        0x00,
        0x00,
    ]);

    Some(pad(data, 20))
}
