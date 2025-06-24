//! Device abstraction layer: re-exports the correct backend at compile time.

mod common;
pub use common::*;

// Feature-gated backends
#[cfg(feature = "libusb")]
mod libusb;
#[cfg(feature = "libusb")]
pub use libusb::Keyboard;

#[cfg(not(feature = "libusb"))]
mod hid;
#[cfg(not(feature = "libusb"))]
pub use hid::Keyboard;
