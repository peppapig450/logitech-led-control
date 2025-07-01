use std::time::Duration;

use super::common::*;
use anyhow::{Result, anyhow};
use rusb::{
    self, Context, DeviceHandle, Direction, Recipient, RequestType, UsbContext, request_type,
};

pub struct Keyboard {
    _ctx: rusb::Context,
    handle: Option<DeviceHandle<Context>>,
    current: Option<DeviceInfo>,
    kernel_detached: bool,
}

fn read_string<T>(handle: &DeviceHandle<T>, index: u8) -> Option<String>
where
    T: rusb::UsbContext,
{
    handle.read_string_descriptor_ascii(index).ok()
}

fn to_device_info<T>(handle: &mut DeviceHandle<T>, desc: &rusb::DeviceDescriptor) -> DeviceInfo
where
    T: rusb::UsbContext,
{
    let get_string = |idx: Option<u8>| match idx {
        Some(i) if i > 0 => read_string(handle, i),
        _ => None,
    };

    let manufacturer = get_string(desc.manufacturer_string_index());
    let product = get_string(desc.product_string_index());
    let serial_number = get_string(desc.serial_number_string_index());

    DeviceInfo {
        vendor_id: desc.vendor_id(),
        product_id: desc.product_id(),
        manufacturer,
        product,
        serial_number,
        model: lookup_model(desc.vendor_id(), desc.product_id()),
    }
}

impl Keyboard {
    /// Enumerate supported keyboards
    pub fn list_keyboards() -> Result<Vec<DeviceInfo>> {
        let ctx = rusb::Context::new()?;
        let mut list = Vec::new();
        for device in ctx.devices()?.iter() {
            let desc = device.device_descriptor()?;
            if lookup_model(desc.vendor_id(), desc.product_id()) == KeyboardModel::Unknown {
                continue;
            }
            if let Ok(mut handle) = device.open() {
                let info = to_device_info(&mut handle, &desc);
                list.push(info);
            }
        }
        Ok(list)
    }

    /// Open a keyboard. If vendor_id or product_id are 0 they are ignored.
    pub fn open(vendor_id: u16, product_id: u16, serial: Option<&str>) -> Result<Self> {
        let ctx = rusb::Context::new()?;
        let mut selected = None;
        let mut device_handle = None;
        for device in ctx.devices()?.iter() {
            let desc = device.device_descriptor()?;
            if lookup_model(desc.vendor_id(), desc.product_id()) == KeyboardModel::Unknown {
                continue;
            }
            if vendor_id != 0 && desc.vendor_id() != vendor_id {
                continue;
            }
            if product_id != 0 && desc.product_id() != product_id {
                continue;
            }
            if let Ok(mut handle) = device.open() {
                let info = to_device_info(&mut handle, &desc);
                if let Some(sn) = serial {
                    if info
                        .serial_number
                        .as_ref()
                        .map(|s| s == sn)
                        .unwrap_or(false)
                    {
                        selected = Some(info);
                        device_handle = Some(handle);
                        break;
                    }
                } else if selected.is_none() {
                    selected = Some(info);
                    device_handle = Some(handle);
                }
            }
        }
        let handle = device_handle.ok_or_else(|| anyhow!("no matching device"))?;
        let info = selected.unwrap();

        if handle.kernel_driver_active(1).unwrap_or(false) {
            handle.detach_kernel_driver(1).ok();
        }
        if let Err(e) = handle.claim_interface(1) {
            return Err(anyhow!("{}", e));
        }
        Ok(Self {
            _ctx: ctx,
            handle: Some(handle),
            current: Some(info),
            kernel_detached: true,
        })
    }

    /// Close the currently open keyboard handle.
    pub fn close(&mut self) -> Result<()> {
        if let Some(h) = self.handle.take() {
            h.release_interface(1).ok();
            if self.kernel_detached {
                h.attach_kernel_driver(1).ok();
            }
        }
        Ok(())
    }

    /// Get information about the currently opened device.
    pub fn current_device(&self) -> Option<&DeviceInfo> {
        self.current.as_ref()
    }

    /// Send a raw HID output report to the keyboard using a USB control transfer.
    ///
    /// This uses the HID class-specific **SET_REPORT (0x09)** request with:
    /// - **wValue** = (`report_type` << 8) \| `report_id`
    /// - `report_type` = **0x02** (*Output Report*)
    /// - `report_id` = **0x12** if `data.len() > 20`, else **0x11**
    ///
    /// These report IDs and behavior are defined by the keyboard's firmware.
    pub fn send_packet(&mut self, data: &[u8]) -> Result<()> {
        let handle = self
            .handle
            .as_mut()
            .ok_or_else(|| anyhow!("no device open"))?;

        let value = if data.len() > 20 { 0x0212 } else { 0x0211 };
        let req_type = request_type(Direction::Out, RequestType::Class, Recipient::Interface);

        handle
            .write_control(req_type, 0x09, value, 1, data, Duration::from_millis(2000))
            .map_err(|e| anyhow!("{}", e))?;

        Ok(())
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        self.close().ok();
        crate::keyboard::model::clear_supported_override();
    }
}
