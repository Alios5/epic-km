use crate::keycode::{code_to_scancode, is_mouse_code};
use crate::profile::{Profile, StickCurve, StickDirection};

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
    /// Previous smoothed stick outputs (used by the smoothing filter)
    pub smooth_rx: f64,
    pub smooth_ry: f64,
    pub smooth_lx: f64,
    pub smooth_ly: f64,
}

/// Stick fraction produced per pixel/second of mouse speed at sensitivity 1.0.
/// ~800 px/s (moderate swipe) therefore reaches full stick deflection.
const MOUSE_SPEED_SCALE: f64 = 0.00125;

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

    // Process right stick from mouse delta. Work in pixels/second so the
    // output depends on physical mouse speed only — not on the polling rate
    // (previously raw per-tick pixels saturated the stick to 100% almost
    // instantly, which made camera movement binary/jerky).
    let hz = profile.right_stick.refresh_interval.max(1) as f64;
    let dt = 1.0 / hz;
    let dx = input.mouse_dx as f64 * hz * MOUSE_SPEED_SCALE;
    let dy = input.mouse_dy as f64 * hz * MOUSE_SPEED_SCALE;
    // Consume deltas
    input.mouse_dx = 0;
    input.mouse_dy = 0;

    let (rx, ry) = process_stick(
        dx,
        dy,
        profile.right_stick.sensitivity,
        profile.right_stick.sensitivity_x,
        profile.right_stick.sensitivity_y,
        profile.right_stick.curve,
        profile.right_stick.deadzone,
        profile.right_stick.invert_x,
        profile.right_stick.invert_y,
    );

    let srx = smooth(input.smooth_rx, rx, profile.right_stick.smoothing, dt);
    let sry = smooth(input.smooth_ry, ry, profile.right_stick.smoothing, dt);
    input.smooth_rx = srx;
    input.smooth_ry = sry;

    state.right_stick_x = (srx * 32767.0) as i16;
    state.right_stick_y = (sry * 32767.0) as i16;

    // Process left stick from keyboard mappings (digital, D-Pad-style)
    let mut ls_x: f64 = 0.0;
    let mut ls_y: f64 = 0.0;

    for mapping in &profile.keyboard_to_left_stick {
        let sc = code_to_scancode(&mapping.key);
        if sc == 0 || is_mouse_code(sc) {
            continue;
        }
        if !input.pressed_keys.contains(&sc) {
            continue;
        }
        match mapping.direction {
            StickDirection::Up => ls_y += 1.0,
            StickDirection::Down => ls_y -= 1.0,
            StickDirection::Left => ls_x -= 1.0,
            StickDirection::Right => ls_x += 1.0,
        }
    }

    // Normalize diagonals to magnitude 1.0
    let ls_mag = (ls_x * ls_x + ls_y * ls_y).sqrt();
    if ls_mag > 1.0 {
        ls_x /= ls_mag;
        ls_y /= ls_mag;
    }

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
