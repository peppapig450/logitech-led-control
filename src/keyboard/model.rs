#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardModel {
    Unknown = 0x00,
    G213,
    G410,
    G413,
    G512,
    G513,
    G610,
    G810,
    G815,
    G910,
    GPro,
}

// Logitech's USB vendor ID (VID) used across all their HID keyboard products.
pub const LOGITECH_VENDOR_ID: u16 = 0x046d;

// Helper macro to define supported keyboards using a shared Logitech vendor ID.
// Saves repetition in the `SUPPORTED_KEYBOARDS` list.
macro_rules! kb {
    ($pid:expr, $model:expr) => {
        (LOGITECH_VENDOR_ID, $pid, $model)
    };
}

/// List of known supported Logitech keyboards, identified by their USB product IDs (PIDs).
/// Note: Some models have multiple PIDs for reasons unknown (blame Logitech). It's probably
/// firmware or regional stuff though.
pub const SUPPORTED_KEYBOARDS: &[(u16, u16, KeyboardModel)] = &[
    kb!(0xc336, KeyboardModel::G213),
    kb!(0xc330, KeyboardModel::G410),
    kb!(0xc33a, KeyboardModel::G413),
    kb!(0xc342, KeyboardModel::G512),
    kb!(0xc33c, KeyboardModel::G513),
    kb!(0xc333, KeyboardModel::G610),
    kb!(0xc338, KeyboardModel::G610),
    kb!(0xc331, KeyboardModel::G810),
    kb!(0xc337, KeyboardModel::G810),
    kb!(0xc33f, KeyboardModel::G815),
    kb!(0xc32b, KeyboardModel::G910),
    kb!(0xc335, KeyboardModel::G910),
    kb!(0xc339, KeyboardModel::GPro), // Covers both G Pro and Pro X
];

// Lookup a model by VID/PID, falls bac kto `Unknown`
pub fn lookup_model(vid: u16, pid: u16) -> KeyboardModel {
    SUPPORTED_KEYBOARDS
        .iter()
        .find_map(|&(v, p, model)| {
            if v == vid && p == pid {
                Some(model)
            } else {
                None
            }
        })
        .unwrap_or(KeyboardModel::Unknown)
}
