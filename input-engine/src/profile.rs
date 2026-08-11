use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardMapping {
    pub id: String,
    pub key: String,
    pub button: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StickDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardStickMapping {
    pub id: String,
    pub key: String,
    pub direction: StickDirection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StickCurve {
    Linear,
    Exponential,
}

/// How a single mouse-driven stick axis behaves.
/// `Analog`: velocity-based, snaps back to 0 when the mouse stops (classic
/// mouse-look, matches the stick's own deadzone/curve/sensitivity).
/// `Gyroscope` (DS4 only): the axis stops driving the stick (which stays
/// centered) and instead feeds the controller's real gyroscope channel —
/// mouse motion becomes an angular rate, like tilting a physical DS4.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AxisInputMode {
    Analog,
    Gyroscope,
}

fn default_axis_mode() -> AxisInputMode {
    AxisInputMode::Analog
}

/// Which virtual controller the engine exposes through ViGEmBus.
/// `Xbox360`: XUSB pad — no motion sensors, but universally supported.
/// `Ds4`: DualShock 4 — buttons/sticks identical, plus gyro/accelerometer
/// channels driven by axes in Gyroscope mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ControllerType {
    #[serde(rename = "xbox360")]
    Xbox360,
    #[serde(rename = "ds4")]
    Ds4,
}

fn default_controller_type() -> ControllerType {
    ControllerType::Xbox360
}

/// Default DS4 gyro pitch rest-offset (raw LSB): the bias declared by
/// ViGEmBus's hardcoded calibration blob (feature report 0x02).
fn default_gyro_bias_pitch() -> i32 {
    1
}

/// Default per-axis sensitivity multiplier (neutral).
fn default_axis_sensitivity() -> f64 {
    1.0
}

/// Cursor hidden during capture by default.
fn default_hide_cursor() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StickConfig {
    pub sensitivity: f64,
    /// Extra per-axis multipliers applied on top of the global sensitivity:
    /// effective_x = raw_x * sensitivity * sensitivity_x (same for Y).
    /// serde defaults keep older profile files valid.
    #[serde(default = "default_axis_sensitivity")]
    pub sensitivity_x: f64,
    #[serde(default = "default_axis_sensitivity")]
    pub sensitivity_y: f64,
    pub curve: StickCurve,
    pub deadzone: f64,
    /// Output smoothing amount (0.0 = off, up to 0.95). Maps to an
    /// exponential time constant, independent of the polling rate.
    #[serde(default)]
    pub smoothing: f64,
    #[serde(rename = "invertY")]
    pub invert_y: bool,
    #[serde(rename = "invertX")]
    pub invert_x: bool,
    pub refresh_interval: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub keyboard_to_button: Vec<KeyboardMapping>,
    pub keyboard_to_left_stick: Vec<KeyboardStickMapping>,
    /// Independent per-axis mode for the mouse-driven right stick: each of
    /// X and Y can be Analog (velocity-based, snaps to 0) or Gyroscope
    /// (accumulated, holds position). serde defaults keep older profile
    /// files valid (both default to Analog, matching prior behavior).
    #[serde(default = "default_axis_mode")]
    pub right_stick_x_mode: AxisInputMode,
    #[serde(default = "default_axis_mode")]
    pub right_stick_y_mode: AxisInputMode,
    pub left_stick: StickConfig,
    pub right_stick: StickConfig,
    pub trigger_threshold: f64,
    pub capture_toggle_key: String,
    /// Hide the OS cursor while capture mode is active.
    /// serde default keeps older profile files valid.
    #[serde(default = "default_hide_cursor")]
    pub hide_cursor: bool,
    /// Which virtual controller to emulate (Xbox 360 or DualShock 4).
    /// serde default keeps older profile files valid (Xbox 360).
    #[serde(default = "default_controller_type")]
    pub controller_type: ControllerType,
    /// Rest-offset compensation added to the DS4 gyroscope channels, in raw
    /// LSB (16 LSB = 1 °/s). Every motion reader subtracts its own assumed
    /// gyro bias (SDL uses the ViGEmBus calibration blob: pitch bias = 1),
    /// so we pre-add it — a perfectly still mouse then reads as exactly
    /// 0 °/s instead of drifting forever. Pitch defaults to 1 (the blob),
    /// yaw to 0; adjust only if the aim still drifts at rest.
    #[serde(default = "default_gyro_bias_pitch")]
    pub gyro_bias_pitch: i32,
    #[serde(default)]
    pub gyro_bias_yaw: i32,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            keyboard_to_button: vec![],
            keyboard_to_left_stick: vec![],
            right_stick_x_mode: AxisInputMode::Analog,
            right_stick_y_mode: AxisInputMode::Analog,
            left_stick: StickConfig {
                sensitivity: 1.0,
                sensitivity_x: 1.0,
                sensitivity_y: 1.0,
                curve: StickCurve::Linear,
                deadzone: 0.1,
                smoothing: 0.0,
                invert_y: false,
                invert_x: false,
                refresh_interval: 60,
            },
            right_stick: StickConfig {
                sensitivity: 1.5,
                sensitivity_x: 1.0,
                sensitivity_y: 1.0,
                curve: StickCurve::Linear,
                deadzone: 0.02,
                smoothing: 0.3,
                invert_y: false,
                invert_x: false,
                refresh_interval: 240,
            },
            trigger_threshold: 0.5,
            capture_toggle_key: "F1".to_string(),
            hide_cursor: true,
            controller_type: ControllerType::Xbox360,
            gyro_bias_pitch: 1,
            gyro_bias_yaw: 0,
        }
    }
}
