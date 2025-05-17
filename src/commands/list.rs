use hidapi::HidApi;

// Scan and print each HID device
pub fn list_keyboards() -> anyhow::Result<()> {
    let api = HidApi::new()?;
    for device in api.device_list() {
        // Only show Logitech VID (0x046D) devices
        if device.vendor_id() == 0x046D {
            println!(
                "{:04x}:{:04x} - {} {} (serial: {:?})",
                device.vendor_id(),
                device.product_id(),
                device.manufacturer_string().unwrap_or_default(),
                device.product_string().unwrap_or_default(),
                device.serial_number(),
            );
        }
    }
    Ok(())
}
