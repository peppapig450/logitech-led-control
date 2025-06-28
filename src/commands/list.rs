use anyhow::Result;
use std::collections::HashSet;

use crate::keyboard::device::Keyboard;

/// List all supported Logitech keyboards, once each.
/// Filters out unknown models and deduplicates by serial or path.
pub fn list_keyboards() -> Result<()> {
    let devices = Keyboard::list_keyboards()?;
    let mut seen = HashSet::new();

    for dev in devices {
        if let Some(serial_num) = &dev.serial_number {
            if !seen.insert(serial_num.clone()) {
                continue;
            }
        }

        println!(
            "{:04x}:{:04x} {:<6?} - {} {} (serial: {:?})",
            dev.vendor_id,
            dev.product_id,
            dev.model,
            dev.manufacturer.as_deref().unwrap_or_default(),
            dev.product.as_deref().unwrap_or_default(),
            dev.serial_number,
        )
    }

    Ok(())
}
