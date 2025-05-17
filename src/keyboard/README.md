# Keyboard Module
> This module provides the core types and logic for Logitech keyboard control.
> It includes device models, packet formatting, effects, and error handling.

### Module structure
```text
src/
└── keyboard/
    ├── mod.rs           // re-exports and public API surface
    ├── model.rs         // KeyboardModel enum + SUPPORTED_KEYBOARDS + lookup_model()
    ├── device.rs        // `struct Keyboard` with open/close, underlying HidApi handle
    ├── packet.rs        // Functions to build raw report packets (e.g. set_key_packet, fx_packet)
    ├── effects.rs       // NativeEffect enum + any higher-level effect logic
    ├── parser.rs        // Conversions from CLI args into Color, Key, EffectPart, etc.
    └── error.rs         // Error type(s) for all keyboard operations
```
