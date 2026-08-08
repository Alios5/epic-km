/// Maps JavaScript KeyboardEvent.code strings to layout-independent key
/// identifiers based on physical key positions (set 1 scan codes).
///
/// Identifier layout (u16):
///   bits 0-7    : set 1 make code
///   bit  8      : E0-extended key (0x100)
///   bit 10      : E1-extended key (0x400, Pause only)
///   0x200-0x204 : pseudo-codes for mouse buttons
///
/// Scan codes are layout-independent, so bindings stay consistent on any
/// keyboard layout (AZERTY, QWERTZ, ...): `KeyW` always means the physical
/// QWERTY-W position, whatever character the key actually produces.
/// Returns 0 if the code is unknown.
pub fn code_to_scancode(code: &str) -> u16 {
    match code {
        // Letters (US QWERTY physical positions)
        "KeyQ" => 0x10, "KeyW" => 0x11, "KeyE" => 0x12, "KeyR" => 0x13,
        "KeyT" => 0x14, "KeyY" => 0x15, "KeyU" => 0x16, "KeyI" => 0x17,
        "KeyO" => 0x18, "KeyP" => 0x19,
        "KeyA" => 0x1E, "KeyS" => 0x1F, "KeyD" => 0x20, "KeyF" => 0x21,
        "KeyG" => 0x22, "KeyH" => 0x23, "KeyJ" => 0x24, "KeyK" => 0x25,
        "KeyL" => 0x26,
        "KeyZ" => 0x2C, "KeyX" => 0x2D, "KeyC" => 0x2E, "KeyV" => 0x2F,
        "KeyB" => 0x30, "KeyN" => 0x31, "KeyM" => 0x32,
        // Digits row
        "Digit1" => 0x02, "Digit2" => 0x03, "Digit3" => 0x04, "Digit4" => 0x05,
        "Digit5" => 0x06, "Digit6" => 0x07, "Digit7" => 0x08, "Digit8" => 0x09,
        "Digit9" => 0x0A, "Digit0" => 0x0B,
        // Function keys
        "F1" => 0x3B, "F2" => 0x3C, "F3" => 0x3D, "F4" => 0x3E,
        "F5" => 0x3F, "F6" => 0x40, "F7" => 0x41, "F8" => 0x42,
        "F9" => 0x43, "F10" => 0x44, "F11" => 0x57, "F12" => 0x58,
        "F13" => 0x64, "F14" => 0x65, "F15" => 0x66, "F16" => 0x67,
        "F17" => 0x68, "F18" => 0x69, "F19" => 0x6A, "F20" => 0x6B,
        "F21" => 0x6C, "F22" => 0x6D, "F23" => 0x6E, "F24" => 0x6F,
        // Modifiers (right-hand variants are E0-extended)
        "ShiftLeft" => 0x2A, "ShiftRight" => 0x36,
        "ControlLeft" => 0x1D, "ControlRight" => 0x11D,
        "AltLeft" => 0x38, "AltRight" => 0x138,
        "MetaLeft" => 0x15B, "MetaRight" => 0x15C,
        // Space, Enter, Backspace, Tab
        "Space" => 0x39, "Enter" => 0x1C, "Backspace" => 0x0E, "Tab" => 0x0F,
        // Escape
        "Escape" => 0x01,
        // Insert, Delete, Home, End, PageUp, PageDown (extended cluster)
        "Insert" => 0x152, "Delete" => 0x153,
        "Home" => 0x147, "End" => 0x14F,
        "PageUp" => 0x149, "PageDown" => 0x151,
        // Arrows (extended)
        "ArrowUp" => 0x148, "ArrowDown" => 0x150,
        "ArrowLeft" => 0x14B, "ArrowRight" => 0x14D,
        // Numpad
        "Numpad0" => 0x52, "Numpad1" => 0x4F, "Numpad2" => 0x50, "Numpad3" => 0x51,
        "Numpad4" => 0x4B, "Numpad5" => 0x4C, "Numpad6" => 0x4D, "Numpad7" => 0x47,
        "Numpad8" => 0x48, "Numpad9" => 0x49,
        "NumpadMultiply" => 0x37, "NumpadAdd" => 0x4E,
        "NumpadSubtract" => 0x4A, "NumpadDecimal" => 0x53,
        "NumpadDivide" => 0x135, "NumpadEnter" => 0x11C,
        // Lock keys
        "CapsLock" => 0x3A, "NumLock" => 0x45, "ScrollLock" => 0x46,
        // PrintScreen (extended), Pause (E1 sequence, make code 0x45)
        "PrintScreen" => 0x137, "Pause" => 0x445,
        // Context menu (extended)
        "ContextMenu" => 0x15D,
        // OEM punctuation (US QWERTY physical positions)
        "Semicolon" => 0x27, "Equal" => 0x0D, "Comma" => 0x33,
        "Minus" => 0x0C, "Period" => 0x34, "Slash" => 0x35,
        "Backquote" => 0x29, "BracketLeft" => 0x1A, "Backslash" => 0x2B,
        "BracketRight" => 0x1B, "Quote" => 0x28,
        // Extra international keys (the <> key on AZERTY, etc.)
        "IntlBackslash" => 0x56, "IntlRo" => 0x73, "IntlYen" => 0x7D,
        // Mouse buttons (custom pseudo-codes, outside scan code ranges)
        "MouseLeft" => 0x200, "MouseRight" => 0x201,
        "MouseMiddle" => 0x202, "MouseX1" => 0x203, "MouseX2" => 0x204,
        _ => 0,
    }
}

/// True if the identifier is a pseudo mouse-button code (not a keyboard scan code).
pub fn is_mouse_code(id: u16) -> bool {
    (0x200..=0x204).contains(&id)
}
