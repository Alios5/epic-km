pub mod profile;
pub mod mapping;
pub mod keycode;
pub mod engine;

#[cfg(target_os = "windows")]
mod windows_capture;

#[cfg(target_os = "windows")]
pub use windows_capture::{start_capture, stop_capture, is_capturing};

#[cfg(not(target_os = "windows"))]
mod stub;

#[cfg(not(target_os = "windows"))]
pub use stub::{start_capture, stop_capture, is_capturing};
