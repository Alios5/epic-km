pub fn start_capture() -> Result<(), String> {
    Err("Raw Input capture is only available on Windows".to_string())
}

pub fn stop_capture() {}

pub fn is_capturing() -> bool {
    false
}
