use super::common::*;
use anyhow::{Result, anyhow};
use hidapi::{HidApi, HidDevice};

fn to_device_info_hid(dev: &hidapi::DeviceInfo) -> DeviceInfo {
    DeviceInfo {
        vendor_id: dev.vendor_id(),
        product_id: dev.product_id(),
        manufacturer: dev.manufacturer_string().map(|s| s.to_owned()),
        product: dev.product_string().map(|s| s.to_owned()),
        serial_number: dev.serial_number().map(|s| s.to_owned()),
        model: lookup_model(dev.vendor_id(), dev.product_id()),
    }
}

pub struct Keyboard {
    _api: HidApi,
    device: Option<HidDevice>,
    current: Option<DeviceInfo>,
}

impl Keyboard {
    /// Enumerate supported keyboards.
    pub fn list_keyboards() -> Result<Vec<DeviceInfo>> {
        let api = HidApi::new()?;
        let devices = api
            .device_list()
            .filter(|d| lookup_model(d.vendor_id(), d.product_id()) != KeyboardModel::Unknown)
            .map(to_device_info_hid)
            .collect();
        Ok(devices)
    }

    /// Open a keyboard. If `vendor_id` or `product_id` are 0 they are ignored.
    pub fn open(vendor_id: u16, product_id: u16, serial: Option<&str>) -> Result<Self> {
        let api = HidApi::new()?;
        let devices = api
            .device_list()
            .filter(|d| lookup_model(d.vendor_id(), d.product_id()) != KeyboardModel::Unknown)
            .filter(|d| {
                (vendor_id == 0 || d.vendor_id() == vendor_id)
                    && (product_id == 0 || d.product_id() == product_id)
            })
            .collect::<Vec<_>>();

        let dev_info = if let Some(sn) = serial {
            devices
                .into_iter()
                .find(|d| d.serial_number().map(|s| s == sn).unwrap_or(false))
        } else {
            devices.into_iter().next()
        }
        .ok_or_else(|| anyhow!("No matching device"))?;

        let device = api.open_path(dev_info.path())?;
        let info = to_device_info_hid(dev_info);
        Ok(Self {
            _api: api,
            device: Some(device),
            current: Some(info),
        })
    }

    /// Close the currently open keyboard handle.
    pub fn close(&mut self) -> Result<()> {
        if let Some(dev) = self.device.take() {
            drop(dev);
        }
        Ok(())
    }

    /// Get information about the currently opened device.
    pub fn current_device(&self) -> Option<&DeviceInfo> {
        self.current.as_ref()
    }

    /// Send a raw HID packet to the keyboard.
    pub fn send_packet(&mut self, data: &[u8]) -> Result<()> {
        let dev = self
            .device
            .as_ref()
            .ok_or_else(|| anyhow!("no device open"))?;

        match data.len() {
            0..=20 => {
                dev.write(data)?;
            }
            64 => {
                dev.write(data)?;
            }
            n => return Err(anyhow!("invalid packet length: {n}")),
        }
        Ok(())
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        self.close().ok();
        crate::keyboard::model::clear_supported_override();
    }
}
