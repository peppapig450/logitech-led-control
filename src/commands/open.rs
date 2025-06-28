use anyhow::Result;

use crate::keyboard::device::Keyboard;

/// Try to open a device by serial (or pick the first one) and print its details
pub fn print_device(serial: Option<&str>) -> Result<()> {
    let mut kbd = Keyboard::open(0, 0, serial)?;

    if let Some(info) = kbd.current_device() {
        println!("Opened device:");
        println!(
            "  VID: {:04x}, PID: {:04x}",
            info.vendor_id, info.product_id
        );
        println!("  Model: {:?}", info.model);
        println!(
            "  Manufacturer: {}",
            info.manufacturer.as_deref().unwrap_or_default()
        );
        println!("  Product: {}", info.product.as_deref().unwrap_or_default());
        println!("  Serial: {:?}", info.serial_number);
    }

    kbd.close()?;
    Ok(())
}
