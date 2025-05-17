use std::collections::HashSet;
use anyhow::Result;
use hidapi::HidApi;

use crate::keyboard::{KeyboardModel, lookup_model};

/// List all supported Logitech keyboards, once each.
/// Fitlers out unknown models and deduplicates by serial or path.
pub fn list_keyboards() -> Result<()> {
    let api = HidApi::new()?;
    let mut seen = HashSet::new();

    for dev in api.device_list() {
        let vid = dev.vendor_id();
        let pid = dev.product_id();

        // Map VID/PID to our enum; skip unsupported devices
        let model = lookup_model(vid, pid);
        if model == KeyboardModel::Unknown {
            continue;
        }

        // Choose a stable dedupe key: prefer serial, else use the raw path
        let key = dev
            .serial_number()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| dev.path().to_string_lossy().into_owned());

        // If we've already printed this one, skip it
        if !seen.insert(key) {
            continue;
        }

        println!(
            "{:04x}:{:04x} {:<6?} - {} {} (serial: {:?})",
            vid,
            pid,
            model,
            dev.manufacturer_string().unwrap_or_default(),
            dev.product_string().unwrap_or_default(),
            dev.serial_number(),
        );
    }

    Ok(())
}
