use crate::keycode::{code_to_scancode, is_mouse_code};
use crate::profile::{AxisInputMode, ControllerType, Profile, StickCurve, StickDirection};

/// Gamepad button bitflags matching XUSB_REPORT buttons.
#[derive(Debug, Clone, Copy, Default)]
pub struct GamepadButtons {
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub start: bool,
    pub back: bool,
    pub left_thumb: bool,
    pub right_thumb: bool,
    pub left_shoulder: bool,
    pub right_shoulder: bool,
    pub left_trigger: bool,
    pub right_trigger: bool,
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
}

/// Output state for the virtual gamepad.
#[derive(Debug, Clone, Copy, Default)]
pub struct GamepadState {
    pub buttons: GamepadButtons,
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub left_stick_x: i16,
    pub left_stick_y: i16,
    pub right_stick_x: i16,
    pub right_stick_y: i16,
    /// DS4 gyroscope output in raw units (16 LSB per °/s), fed by
    /// mouse-driven axes in Gyroscope mode. Zero = controller not rotating.
    /// Ignored by the Xbox 360 target, which has no motion channels.
    pub gyro_pitch: i16, // report gyro_x
    pub gyro_yaw: i16,   // report gyro_y
}

/// Raw input state from the capture thread.
#[derive(Debug, Default)]
pub struct RawInputState {
    /// Set of currently pressed key identifiers (scan codes, see keycode.rs)
    pub pressed_keys: std::collections::HashSet<u16>,
    /// Accumulated mouse delta X since last read
    pub mouse_dx: i32,
    /// Accumulated mouse delta Y since last read
    pub mouse_dy: i32,
    /// Mouse button states (bit 0 = left, 1 = right, 2 = middle, 3 = x1, 4 = x2)
    pub mouse_buttons: u8,
    /// Previous smoothed stick outputs (used by the smoothing filter, Analog mode)
    pub smooth_rx: f64,
    pub smooth_ry: f64,
    pub smooth_lx: f64,
    pub smooth_ly: f64,
}

/// Stick fraction produced per pixel/second of mouse speed at sensitivity 1.0.
/// ~800 px/s (moderate swipe) therefore reaches full stick deflection.
const MOUSE_SPEED_SCALE: f64 = 0.00125;

/// Degrees of gyroscope rotation reported per raw mouse pixel at
/// sensitivity 1.0. Each report carries the rotation accumulated during
/// its tick converted to a per-second rate, so the *integral* of the gyro
/// output over time always equals the total mouse travel — the burstiness
/// of the OS mouse stream doesn't distort the aim.
const GYRO_DEG_PER_PX: f64 = 0.05;

/// DS4 gyroscope raw unit: 16 LSB per degree/second. Matches the factory
/// defaults every reader falls back to (Linux hid-playstation:
/// DS4_GYRO_RES_PER_DEG_S; SDL: gyro_numerator/denominator = 1/16).
const DS4_GYRO_LSB_PER_DPS: f64 = 16.0;

/// Exponential smoothing towards the target, independent of the polling rate.
/// `amount` (0.0 = off .. 0.95) maps to a time constant of amount * 0.25 s.
fn smooth(prev: f64, target: f64, amount: f64, dt: f64) -> f64 {
    if amount <= 0.0 {
        return target;
    }
    let tau = amount * 0.25;
    let step = 1.0 - (-dt / tau).exp();
    let out = prev + (target - prev) * step;
    // Snap when close enough: the stick must reach exactly 0 (no residual
    // camera drift) and exactly the target.
    if (out - target).abs() < 0.002 { target } else { out }
}

/// Maps a button name string to the corresponding GamepadButtons field.
fn set_button(buttons: &mut GamepadButtons, name: &str, pressed: bool) {
    match name {
        "A" => buttons.a = pressed,
        "B" => buttons.b = pressed,
        "X" => buttons.x = pressed,
        "Y" => buttons.y = pressed,
        "DPadUp" => buttons.dpad_up = pressed,
        "DPadDown" => buttons.dpad_down = pressed,
        "DPadLeft" => buttons.dpad_left = pressed,
        "DPadRight" => buttons.dpad_right = pressed,
        "LB" => buttons.left_shoulder = pressed,
        "RB" => buttons.right_shoulder = pressed,
        "LT" => buttons.left_trigger = pressed,
        "RT" => buttons.right_trigger = pressed,
        "Start" => buttons.start = pressed,
        "Back" => buttons.back = pressed,
        "LeftThumb" => buttons.left_thumb = pressed,
        "RightThumb" => buttons.right_thumb = pressed,
        _ => {}
    }
}

/// Processes a single mouse-driven axis in Analog mode: velocity-based,
/// with its own deadzone/curve/sensitivity, snapping back to 0 once the
/// mouse stops (the smoothing filter is applied by the caller).
fn process_axis_analog(
    raw: f64,
    sensitivity: f64,
    axis_sensitivity: f64,
    curve: StickCurve,
    deadzone: f64,
    invert: bool,
) -> f64 {
    let mut v = raw * sensitivity * axis_sensitivity;

    let mag = v.abs();
    if mag < deadzone {
        return 0.0;
    }
    let sign = if v < 0.0 { -1.0 } else { 1.0 };
    let rescaled = (mag - deadzone) / (1.0 - deadzone);
    v = sign * rescaled;

    v = match curve {
        StickCurve::Linear => v,
        StickCurve::Exponential => {
            let s = if v < 0.0 { -1.0 } else { 1.0 };
            s * v.abs().powi(2)
        }
    };

    v = v.clamp(-1.0, 1.0);
    if invert { v = -v; }
    v
}

/// Processes a mouse-driven axis in Gyroscope mode: converts the raw pixel
/// delta of this tick into a DS4 gyroscope angular rate, in raw units
/// (16 LSB per °/s). When the mouse stops the rate is zero — like a real
/// gyroscope, which only reports while the controller is rotating.
fn process_axis_gyro(
    raw_pixels: f64,
    sensitivity: f64,
    axis_sensitivity: f64,
    invert: bool,
    hz: f64,
) -> i16 {
    let mut degrees = raw_pixels * GYRO_DEG_PER_PX * sensitivity * axis_sensitivity;
    if invert { degrees = -degrees; }
    let dps = degrees * hz;
    (dps * DS4_GYRO_LSB_PER_DPS).clamp(i16::MIN as f64, i16::MAX as f64) as i16
}

/// Applies stick processing: global + per-axis sensitivity, deadzone, curve,
/// inversions. Returns (x, y) in range [-1.0, 1.0].
fn process_stick(
    raw_x: f64,
    raw_y: f64,
    sensitivity: f64,
    sensitivity_x: f64,
    sensitivity_y: f64,
    curve: StickCurve,
    deadzone: f64,
    invert_x: bool,
    invert_y: bool,
) -> (f64, f64) {
    let mut x = raw_x * sensitivity * sensitivity_x;
    let mut y = raw_y * sensitivity * sensitivity_y;

    // Apply deadzone
    let magnitude = (x * x + y * y).sqrt();
    if magnitude < deadzone {
        return (0.0, 0.0);
    }

    // Rescale past deadzone
    let rescaled = (magnitude - deadzone) / (1.0 - deadzone);
    let scale = rescaled / magnitude;
    x *= scale;
    y *= scale;

    // Apply curve
    let apply_curve = |v: f64| match curve {
        StickCurve::Linear => v,
        StickCurve::Exponential => {
            let sign = if v < 0.0 { -1.0 } else { 1.0 };
            sign * v.abs().powi(2)
        }
    };
    x = apply_curve(x);
    y = apply_curve(y);

    // Clamp
    x = x.clamp(-1.0, 1.0);
    y = y.clamp(-1.0, 1.0);

    // Apply inversions
    if invert_x { x = -x; }
    if invert_y { y = -y; }

    (x, y)
}

/// Maps raw input state + profile to gamepad output state.
/// Consumes the accumulated mouse delta.
pub fn map_input(
    input: &mut RawInputState,
    profile: &Profile,
) -> GamepadState {
    let mut state = GamepadState::default();

    // Map keyboard keys to buttons
    for mapping in &profile.keyboard_to_button {
        let sc = code_to_scancode(&mapping.key);
        if sc == 0 || is_mouse_code(sc) {
            continue;
        }
        let pressed = input.pressed_keys.contains(&sc);
        set_button(&mut state.buttons, &mapping.button, pressed);
    }

    // Map mouse buttons to buttons
    for mapping in &profile.keyboard_to_button {
        let sc = code_to_scancode(&mapping.key);
        if is_mouse_code(sc) {
            let bit = (sc - 0x200) as u8;
            let pressed = (input.mouse_buttons & (1 << bit)) != 0;
            set_button(&mut state.buttons, &mapping.button, pressed);
        }
    }

    // Read raw mouse delta (pixels) once, then derive both the velocity
    // form (for Analog axes) and keep the raw pixel form (for Gyroscope
    // axes), before consuming the accumulator.
    let hz = profile.right_stick.refresh_interval.max(1) as f64;
    let dt = 1.0 / hz;
    let raw_dx = input.mouse_dx as f64;
    let raw_dy = input.mouse_dy as f64;
    let vel_dx = raw_dx * hz * MOUSE_SPEED_SCALE;
    let vel_dy = raw_dy * hz * MOUSE_SPEED_SCALE;
    // Consume deltas
    input.mouse_dx = 0;
    input.mouse_dy = 0;

    // Right stick: each axis independently is Analog (velocity-based,
    // snaps back to 0) or Gyroscope (drives the DS4's real gyro channel
    // instead of the stick, which then stays centered).
    // Gyroscope mode only applies to the DS4 target — an XUSB pad has no
    // motion channels, so both axes fall back to Analog in Xbox 360 mode
    // (the UI also hides these selectors unless DS4 is selected).
    let (x_mode, y_mode) = if profile.controller_type == ControllerType::Ds4 {
        (profile.right_stick_x_mode, profile.right_stick_y_mode)
    } else {
        (AxisInputMode::Analog, AxisInputMode::Analog)
    };
    let rx = match x_mode {
        AxisInputMode::Analog => {
            let target = process_axis_analog(
                vel_dx,
                profile.right_stick.sensitivity,
                profile.right_stick.sensitivity_x,
                profile.right_stick.curve,
                profile.right_stick.deadzone,
                profile.right_stick.invert_x,
            );
            let smoothed = smooth(input.smooth_rx, target, profile.right_stick.smoothing, dt);
            input.smooth_rx = smoothed;
            smoothed
        }
        AxisInputMode::Gyroscope => {
            // Mouse X drives the DS4's yaw gyroscope channel (report
            // gyro_y); the stick axis itself stays centered.
            state.gyro_yaw = process_axis_gyro(
                raw_dx,
                profile.right_stick.sensitivity,
                profile.right_stick.sensitivity_x,
                profile.right_stick.invert_x,
                hz,
            );
            0.0
        }
    };

    let ry = match y_mode {
        AxisInputMode::Analog => {
            let target = process_axis_analog(
                vel_dy,
                profile.right_stick.sensitivity,
                profile.right_stick.sensitivity_y,
                profile.right_stick.curve,
                profile.right_stick.deadzone,
                profile.right_stick.invert_y,
            );
            let smoothed = smooth(input.smooth_ry, target, profile.right_stick.smoothing, dt);
            input.smooth_ry = smoothed;
            smoothed
        }
        AxisInputMode::Gyroscope => {
            // Mouse Y drives the DS4's pitch gyroscope channel (report
            // gyro_x). On a real DS4 a positive pitch rate is the nose
            // tilting up, while mouse-up arrives as negative deltas —
            // hence the sign flip (invert_y flips it back if needed).
            state.gyro_pitch = process_axis_gyro(
                -raw_dy,
                profile.right_stick.sensitivity,
                profile.right_stick.sensitivity_y,
                profile.right_stick.invert_y,
                hz,
            );
            0.0
        }
    };

    state.right_stick_x = (rx * 32767.0) as i16;
    state.right_stick_y = (ry * 32767.0) as i16;

    // Left stick: always digital keyboard directions (D-Pad-style).
    let (ls_x, ls_y) = digital_stick(&profile.keyboard_to_left_stick, &input.pressed_keys);

    let (lx, ly) = process_stick(
        ls_x,
        ls_y,
        profile.left_stick.sensitivity,
        profile.left_stick.sensitivity_x,
        profile.left_stick.sensitivity_y,
        profile.left_stick.curve,
        profile.left_stick.deadzone,
        profile.left_stick.invert_x,
        profile.left_stick.invert_y,
    );

    let slx = smooth(input.smooth_lx, lx, profile.left_stick.smoothing, dt);
    let sly = smooth(input.smooth_ly, ly, profile.left_stick.smoothing, dt);
    input.smooth_lx = slx;
    input.smooth_ly = sly;

    state.left_stick_x = (slx * 32767.0) as i16;
    state.left_stick_y = (sly * 32767.0) as i16;

    state
}

/// Computes a digital (D-Pad-style) stick axis from keyboard direction
/// mappings. Diagonals are normalized to magnitude 1.0.
fn digital_stick(
    mappings: &[crate::profile::KeyboardStickMapping],
    pressed_keys: &std::collections::HashSet<u16>,
) -> (f64, f64) {
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;

    for mapping in mappings {
        let sc = code_to_scancode(&mapping.key);
        if sc == 0 || is_mouse_code(sc) {
            continue;
        }
        if !pressed_keys.contains(&sc) {
            continue;
        }
        match mapping.direction {
            StickDirection::Up => y += 1.0,
            StickDirection::Down => y -= 1.0,
            StickDirection::Left => x -= 1.0,
            StickDirection::Right => x += 1.0,
        }
    }

    let mag = (x * x + y * y).sqrt();
    if mag > 1.0 {
        x /= mag;
        y /= mag;
    }
    (x, y)
}
