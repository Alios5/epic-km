//! Virtual Xbox 360 gamepad output for Linux, via the kernel `uinput`
//! module. This is the Linux counterpart to the ViGEmBus path used on
//! Windows (see `engine.rs`): same `GamepadState` input, same axis range
//! (`i16`), same button semantics — only the transport differs.
//!
//! Requires read/write access to `/dev/uinput` (usually via the `input`
//! group or a udev rule; see project README for setup instructions).

use crate::mapping::GamepadState;
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent, Key, UinputAbsSetup};
use std::io;

/// Wraps a uinput virtual device configured to look like an Xbox 360
/// controller (same button/axis layout consumers expect from XInput).
pub struct LinuxGamepad {
    device: VirtualDevice,
}

/// Maps `GamepadButtons` fields to Linux `Key` codes using the standard
/// gamepad button codes (`BTN_SOUTH` etc.), matching the Xbox 360 layout
/// via the SDL/kernel convention (A=SOUTH, B=EAST, X=NORTH, Y=WEST).
const BUTTON_CODES: &[Key] = &[
    Key::BTN_SOUTH,   // A
    Key::BTN_EAST,    // B
    Key::BTN_NORTH,   // X
    Key::BTN_WEST,    // Y
    Key::BTN_TL,      // left shoulder
    Key::BTN_TR,      // right shoulder
    Key::BTN_SELECT,  // back
    Key::BTN_START,   // start
    Key::BTN_THUMBL,  // left thumb click
    Key::BTN_THUMBR,  // right thumb click
    Key::BTN_DPAD_UP,
    Key::BTN_DPAD_DOWN,
    Key::BTN_DPAD_LEFT,
    Key::BTN_DPAD_RIGHT,
];

fn axis_setup(code: AbsoluteAxisType, min: i32, max: i32) -> UinputAbsSetup {
    UinputAbsSetup::new(code, AbsInfo::new(0, min, max, 16, 128, 1))
}

impl LinuxGamepad {
    /// Creates and registers the virtual device with the kernel. Fails if
    /// `/dev/uinput` cannot be opened (missing permissions/module).
    pub fn new() -> io::Result<Self> {
        let mut keys: AttributeSet<Key> = AttributeSet::new();
        for &code in BUTTON_CODES {
            keys.insert(code);
        }

        let device = VirtualDeviceBuilder::new()?
            .name("Epic KM Virtual Xbox 360 Controller")
            .with_keys(&keys)?
            .with_absolute_axis(&axis_setup(AbsoluteAxisType::ABS_X, i16::MIN as i32, i16::MAX as i32))?
            .with_absolute_axis(&axis_setup(AbsoluteAxisType::ABS_Y, i16::MIN as i32, i16::MAX as i32))?
            .with_absolute_axis(&axis_setup(AbsoluteAxisType::ABS_RX, i16::MIN as i32, i16::MAX as i32))?
            .with_absolute_axis(&axis_setup(AbsoluteAxisType::ABS_RY, i16::MIN as i32, i16::MAX as i32))?
            .with_absolute_axis(&axis_setup(AbsoluteAxisType::ABS_Z, 0, 255))?
            .with_absolute_axis(&axis_setup(AbsoluteAxisType::ABS_RZ, 0, 255))?
            .build()?;

        Ok(Self { device })
    }

    /// Pushes a full `GamepadState` snapshot to the kernel device. Mirrors
    /// the report-building logic in `engine.rs`'s Windows emission thread.
    pub fn update(&mut self, state: &GamepadState) -> io::Result<()> {
        let b = &state.buttons;
        let button_states: [(Key, bool); 14] = [
            (Key::BTN_SOUTH, b.a),
            (Key::BTN_EAST, b.b),
            (Key::BTN_NORTH, b.x),
            (Key::BTN_WEST, b.y),
            (Key::BTN_TL, b.left_shoulder),
            (Key::BTN_TR, b.right_shoulder),
            (Key::BTN_SELECT, b.back),
            (Key::BTN_START, b.start),
            (Key::BTN_THUMBL, b.left_thumb),
            (Key::BTN_THUMBR, b.right_thumb),
            (Key::BTN_DPAD_UP, b.dpad_up),
            (Key::BTN_DPAD_DOWN, b.dpad_down),
            (Key::BTN_DPAD_LEFT, b.dpad_left),
            (Key::BTN_DPAD_RIGHT, b.dpad_right),
        ];

        let mut events: Vec<InputEvent> = Vec::with_capacity(20);
        for (code, pressed) in button_states {
            events.push(InputEvent::new(EventType::KEY, code.code(), pressed as i32));
        }

        events.push(InputEvent::new(
            EventType::ABSOLUTE,
            AbsoluteAxisType::ABS_X.0,
            state.left_stick_x as i32,
        ));
        events.push(InputEvent::new(
            EventType::ABSOLUTE,
            AbsoluteAxisType::ABS_Y.0,
            state.left_stick_y as i32,
        ));
        events.push(InputEvent::new(
            EventType::ABSOLUTE,
            AbsoluteAxisType::ABS_RX.0,
            state.right_stick_x as i32,
        ));
        events.push(InputEvent::new(
            EventType::ABSOLUTE,
            AbsoluteAxisType::ABS_RY.0,
            state.right_stick_y as i32,
        ));
        events.push(InputEvent::new(
            EventType::ABSOLUTE,
            AbsoluteAxisType::ABS_Z.0,
            if b.left_trigger { 255 } else { 0 },
        ));
        events.push(InputEvent::new(
            EventType::ABSOLUTE,
            AbsoluteAxisType::ABS_RZ.0,
            if b.right_trigger { 255 } else { 0 },
        ));

        self.device.emit(&events)
    }
}
