use core::time::Duration;
use strum_macros::{Display, EnumString};

use crate::keyboard::{
    Color, KeyboardModel,
    packet::{self},
};

type Packet = Vec<u8>;
type Packets = Vec<Packet>;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
#[strum(ascii_case_insensitive, serialize_all = "kebab-case")]
pub enum NativeEffectGroup {
    Off,
    Color,
    Breathing,
    Cycle,
    Waves,
    Ripple,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
#[strum(ascii_case_insensitive, serialize_all = "kebab-case")]
pub enum NativeEffect {
    Off = 0,
    Color = (NativeEffectGroup::Color as u16) << 8,
    Breathing = (NativeEffectGroup::Breathing as u16) << 8,
    Cycle = (NativeEffectGroup::Cycle as u16) << 8,
    Waves = (NativeEffectGroup::Waves as u16) << 8,
    HWave,
    VWave,
    CWave,
    Ripple = (NativeEffectGroup::Ripple as u16) << 8,
}

impl NativeEffect {
    /// Extract the high-byte **group** for quick pattern matches
    #[inline]
    const fn group(self) -> NativeEffectGroup {
        // Safety: the first 8 bits of every `NativeEffect` encode its group.
        unsafe { core::mem::transmute::<u8, NativeEffectGroup>((self as u16 >> 8) as u8) }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
#[strum(ascii_case_insensitive, serialize_all = "kebab-case")]
pub enum NativeEffectPart {
    All = 0xff,
    Keys = 0x00,
    Logo,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
#[strum(ascii_case_insensitive, serialize_all = "kebab-case")]
pub enum NativeEffectStorage {
    None = 0x00,
    /// User stored effect recalled with backlight+7
    User,
}

/// Translate a lighting effect into one or more HID packets.
///
/// *Returns*
/// `Some(vec![])`&nbsp;- the combination is valid but no packet is required (e.g. logo part on G213).
/// `None` — the combination is unsupported.
pub fn native_effect_packets(
    model: KeyboardModel,
    effect: NativeEffect,
    part: NativeEffectPart,
    period: Duration,
    color: Color,
    storage: NativeEffectStorage,
) -> Option<Packets> {
    // 1. Expand the virtual "All" part
    if part == NativeEffectPart::All {
        return [NativeEffectPart::Keys, NativeEffectPart::Logo]
            .into_iter()
            .filter_map(|p| native_effect_packets(model, effect, p, period, color, storage))
            .flatten()
            .collect::<Packets>()
            .into();
    }

    // 2. Short-circuit: logo LEDs absent on these boards
    if matches!(model, KeyboardModel::G213 | KeyboardModel::G413) && part == NativeEffectPart::Logo
    {
        return Some(Packets::new());
    }

    // 3. Base payload - bail if unsupported
    let mut data = packet::native_effect_packet(model, effect, part, period, color, storage)?;

    let mut packets = Packets::new();

    // 4. Model-specific tweaks
    match model {
        KeyboardModel::G815 => {
            // The G815 expects a 20-byte setup header first.
            let mut setup = [0u8; 20];
            setup[..7].copy_from_slice(&[0x11, 0xff, 0x0f, 0x5c, 0x01, 0x03, 0x03]);
            packets.push(setup.to_vec());

            data[16] = 0x01; // Common footer byte for G815

            match part {
                NativeEffectPart::Keys => {
                    data[4] = 0x01;
                    if effect == NativeEffect::Ripple {
                        // Keys ripple encodes the *period* explicitly
                        let per_ms: u16 = period.as_millis().try_into().unwrap_or(u16::MAX);
                        data[9] = 0x00;
                        data[10] = (per_ms >> 8) as u8;
                        data[11] = (per_ms & 0xff) as u8;
                        data[12] = 0x00;
                    }
                }
                NativeEffectPart::Logo => {
                    data[4] = 0x00;
                    data[5] = match effect {
                        NativeEffect::Breathing => 0x03,
                        NativeEffect::CWave | NativeEffect::VWave | NativeEffect::HWave => {
                            data[13] = 0x64;
                            0x02
                        }
                        NativeEffect::Waves | NativeEffect::Cycle => 0x02,
                        NativeEffect::Ripple | NativeEffect::Off => 0x00,
                        _ => 0x01,
                    };
                }
                _ => {}
            }
        }

        // 4.b Everything else
        _ => {
            // Waves on the logo fal back to a static cyan color.
            if effect.group() == NativeEffectGroup::Waves && part == NativeEffectPart::Logo {
                const CYAN: Color = Color {
                    red: 0x00,
                    green: 0xff,
                    blue: 0xff,
                };
                return native_effect_packets(
                    model,
                    NativeEffect::Color,
                    part,
                    Duration::ZERO,
                    CYAN,
                    storage,
                );
            }
        }
    }

    // 5. Final payload
    packets.push(data);
    Some(packets)
}
