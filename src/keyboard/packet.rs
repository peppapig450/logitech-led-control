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

/// Packet used to commit changes to the device.
pub fn commit_packet(model: KeyboardModel) -> Option<Vec<u8>> {
    model
        .spec()
        .commit
        .map(|commit_bytes| pad(commit_bytes.to_vec(), 20))
}

/// Raw HID header for a key group.
fn group_address(model: KeyboardModel, group: u8) -> Option<Packet> {
    model
        .spec()
        .group_addresses
        .iter()
        .find(|&&(test_group, _)| test_group == group)
        .map(|&(_, packet)| packet)
}

/// Translate a [`Key`] into the byte identifier used by the G815.
fn g815_key_id(key: Key) -> Option<u8> {
    let low = key.hid_code();

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
            let header = model.spec().keys_header?;
            data.extend_from_slice(header);
            data.extend_from_slice(&[color.red, color.green, color.blue]);

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
                    kv.key.hid_code(),
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
    let header = model.spec().region_header?;
    Some(pad(
        [header, &[region, 0x01, color.red, color.green, color.blue]].concat(),
        20,
    ))
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

    let (p0, p1) = model.spec().effect_params?;

    let per_ms: u16 = period.as_millis().try_into().unwrap_or(u16::MAX);
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
