use anyhow::{anyhow, Result};
use hidapi::HidApi;

use crate::keyboard::{model::lookup_model, KeyboardModel};

/// Try to open a device by serial (or pick the first one) and print its details
pub fn print_device(serial: Option<String>) -> Result<()> {
    let api = HidApi::new()?;

    // Collect all supported devices
    let mut devices = api
        .device_list()
        .filter(|dev| lookup_model(dev.vendor_id(), dev.product_id()) != KeyboardModel::Unknown)
        .collect::<Vec<_>>();

    if devices.is_empty() {
        return Err(anyhow!("No supported Logitech keyboards found"));
    }

    // Choose the device
    let device_info = if let Some(serial_str) = serial {
        devices
            .into_iter()
            .find(|d| d.serial_number().as_deref() == Some(serial_str.as_str()))
            .ok_or_else(|| anyhow!("No device with serial `{}` found", serial_str))?
    } else {
        // Fallback to the first supported device
        devices.remove(0)
    };

    // Open the device
    let _device = api.open_path(device_info.path())
        .map_err(|e| anyhow!("Failed to open device: {}", e))?;

    // Print out some info about the device
    println!("Opened device:");
    println!("  VID: {:04x}, PID: {:04x}", device_info.vendor_id(), device_info.product_id());
    println!("  Model: {:?}", lookup_model(device_info.vendor_id(), device_info.product_id()));
    println!("  Manufacturer: {}", device_info.manufacturer_string().unwrap_or_default());
    println!("  Product: {}", device_info.product_string().unwrap_or_default());
    println!("  Serial:  {:?}", device_info.serial_number());

    Ok(())
}