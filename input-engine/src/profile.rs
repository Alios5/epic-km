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

/// Default per-axis sensitivity multiplier (neutral).
fn default_axis_sensitivity() -> f64 {
    1.0
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
    pub left_stick: StickConfig,
    pub right_stick: StickConfig,
    pub trigger_threshold: f64,
    pub capture_toggle_key: String,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            keyboard_to_button: vec![],
            keyboard_to_left_stick: vec![],
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
        }
    }
}
