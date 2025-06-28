use crate::keyboard::{
    self as keyboard, Color, KeyGroup, KeyValue, KeyboardModel, NativeEffect, NativeEffectPart,
    NativeEffectStorage, OnBoardMode, StartupMode,
};
use anyhow::{Result, anyhow};
use core::time::Duration;
use std::collections::BTreeMap;
use strum::IntoEnumIterator;

/// High level keyboard operations.
///
/// These are stubs for now so that the profile parser can call a uniform API.
#[allow(dead_code)]
pub trait KeyboardApi {
    fn commit(&mut self) -> Result<()> {
        Ok(())
    }

    fn set_all_keys(&mut self, _color: Color) -> Result<()> {
        Ok(())
    }

    fn set_group_keys(&mut self, _group: KeyGroup, _color: Color) -> Result<()> {
        Ok(())
    }

    fn set_keys(&mut self, _keys: &[KeyValue]) -> Result<()> {
        Ok(())
    }

    fn set_region(&mut self, _region: u8, _color: Color) -> Result<()> {
        Ok(())
    }

    fn set_mr_key(&mut self, _value: u8) -> Result<()> {
        Ok(())
    }

    fn set_mn_key(&mut self, _value: u8) -> Result<()> {
        Ok(())
    }

    fn set_gkeys_mode(&mut self, _value: u8) -> Result<()> {
        Ok(())
    }

    fn set_startup_mode(&mut self, _mode: StartupMode) -> Result<()> {
        Ok(())
    }

    fn set_on_board_mode(&mut self, _mode: OnBoardMode) -> Result<()> {
        Ok(())
    }

    fn set_fx(
        &mut self,
        _effect: NativeEffect,
        _part: NativeEffectPart,
        _period: Duration,
        _color: Color,
        _storage: NativeEffectStorage,
    ) -> Result<()> {
        Ok(())
    }
}

impl KeyboardApi for crate::keyboard::device::Keyboard {
    fn commit(&mut self) -> Result<()> {
        let model = self
            .current_device()
            .ok_or_else(|| anyhow!("no device open"))?
            .model;

        if let Some(packet) = keyboard::packet::commit_packet(model) {
            self.send_packet(&packet)?;
        }

        Ok(())
    }

    fn set_keys(&mut self, keys: &[KeyValue]) -> Result<()> {
        if keys.is_empty() {
            return Ok(());
        }

        let model = self
            .current_device()
            .ok_or_else(|| anyhow!("no device open"))?
            .model;

        match model {
            KeyboardModel::G213 | KeyboardModel::G413 => return Ok(()),
            KeyboardModel::G815 => {
                let mut by_color: BTreeMap<(u8, u8, u8), Vec<KeyValue>> = BTreeMap::new();
                for &kv in keys {
                    by_color
                        .entry((kv.color.red, kv.color.green, kv.color.blue))
                        .or_default()
                        .push(kv);
                }

                for vals in by_color.values() {
                    for chunk in vals.chunks(13) {
                        if let Some(packet) = keyboard::packet::set_keys_packet(model, chunk) {
                            self.send_packet(&packet)?;
                        }
                    }
                }
            }
            _ => {
                let mut by_group: BTreeMap<u8, Vec<KeyValue>> = BTreeMap::new();
                for &kv in keys {
                    by_group.entry(kv.key.group()).or_default().push(kv);
                }

                for (group, vals) in by_group {
                    let size = if group == 0 { 20 } else { 64 };
                    let max_keys = (size - 8) / 4;

                    for chunk in vals.chunks(max_keys) {
                        if let Some(packet) = keyboard::packet::set_keys_packet(model, chunk) {
                            self.send_packet(&packet)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn set_group_keys(&mut self, group: KeyGroup, color: Color) -> Result<()> {
        let keys: Vec<KeyValue> = group.keys().map(|k| KeyValue { key: k, color }).collect();

        self.set_keys(&keys)
    }

    fn set_all_keys(&mut self, color: Color) -> Result<()> {
        for group in KeyGroup::iter() {
            self.set_group_keys(group, color)?;
        }
        Ok(())
    }
}
