use crate::keyboard::{
    self as keyboard, Color, KeyGroup, KeyValue, NativeEffect, NativeEffectPart,
    NativeEffectStorage, OnBoardMode, StartupMode,
};
use anyhow::{Result, anyhow};
use core::time::Duration;

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
}
