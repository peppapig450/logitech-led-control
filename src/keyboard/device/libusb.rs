use super::common::*;
use anyhow::{Result, anyhow};
use rusb::{self, Context, DeviceHandle, UsbContext};

pub struct Keyboard {
    ctx: rusb::Context,
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
            ctx,
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
}
