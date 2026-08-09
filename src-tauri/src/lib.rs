// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use input_engine;
use input_engine::profile::Profile;
use std::fs;
use tauri::{Emitter, Manager};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// ---------------------------------------------------------------------------
// Window state persistence
//
// The window size (physical pixels) is stored in `window-state.json` inside
// the app data directory. It is restored on startup and saved only when the
// size actually changes. Resizes reported while the window is minimized or
// maximized are ignored so the last "normal" size is the one restored.
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq)]
struct WindowSizeState {
    width: u32,
    height: u32,
}

static LAST_SAVED_SIZE: std::sync::Mutex<Option<WindowSizeState>> = std::sync::Mutex::new(None);

fn window_state_path(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    app.path()
        .app_data_dir()
        .ok()
        .map(|dir| dir.join("window-state.json"))
}

fn load_window_size(app: &tauri::AppHandle) -> Option<WindowSizeState> {
    let content = fs::read_to_string(window_state_path(app)?).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_window_size(app: &tauri::AppHandle, size: WindowSizeState) {
    if let Some(path) = window_state_path(app) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&size) {
            let _ = fs::write(path, json);
        }
    }
}

fn profiles_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let dir = app_data.join("profiles");
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create profiles dir: {}", e))?;
    }
    Ok(dir)
}

#[tauri::command]
fn save_profile(app: tauri::AppHandle, name: String, data: serde_json::Value) -> Result<(), String> {
    let dir = profiles_dir(&app)?;
    let safe_name = name.replace('/', "_").replace('\\', "_");
    let path = dir.join(format!("{}.json", safe_name));
    let json = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("Failed to serialize profile: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write profile file: {}", e))
}

#[tauri::command]
fn load_profile(app: tauri::AppHandle, name: String) -> Result<serde_json::Value, String> {
    let dir = profiles_dir(&app)?;
    let safe_name = name.replace('/', "_").replace('\\', "_");
    let path = dir.join(format!("{}.json", safe_name));
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read profile '{}': {}", name, e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse profile '{}': {}", name, e))
}

#[tauri::command]
fn list_profiles(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let dir = profiles_dir(&app)?;
    let mut profiles = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    profiles.push(name.trim_end_matches(".json").to_string());
                }
            }
        }
    }
    profiles.sort();
    Ok(profiles)
}

#[tauri::command]
fn delete_profile(app: tauri::AppHandle, name: String) -> Result<(), String> {
    let dir = profiles_dir(&app)?;
    let safe_name = name.replace('/', "_").replace('\\', "_");
    let path = dir.join(format!("{}.json", safe_name));
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete profile '{}': {}", name, e))
    } else {
        Ok(())
    }
}

/// Export the given profile data to an arbitrary file path (user-chosen via
/// a save dialog on the frontend).
#[tauri::command]
fn export_profile(path: String, data: serde_json::Value) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("Failed to serialize profile: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write profile file: {}", e))
}

#[tauri::command]
fn center_cursor() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SetCursorPos, SM_CXSCREEN, SM_CYSCREEN};
        unsafe {
            let cx = GetSystemMetrics(SM_CXSCREEN);
            let cy = GetSystemMetrics(SM_CYSCREEN);
            SetCursorPos(cx / 2, cy / 2)
                .map_err(|e| format!("SetCursorPos failed: {}", e))
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

#[tauri::command]
fn show_cursor(show: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::ShowCursor;
        unsafe {
            if show {
                while ShowCursor(true) < 0 {}
            } else {
                while ShowCursor(false) >= 0 {}
            }
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

/// Returns the ViGEmBus driver status:
/// - "not-installed": service not found on the system
/// - "not-responding": service exists but the bus rejects clients
///   (typically right after installation, before the required reboot)
/// - "ok": driver present and accepting clients
#[tauri::command]
fn check_vigembus() -> String {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        // sc query ViGEmBus returns exit code 0 if the service exists
        let installed = Command::new("sc")
            .args(["query", "ViGEmBus"])
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false);
        if !installed {
            return "not-installed".to_string();
        }
        // Functional check: can we actually open a bus connection?
        if input_engine::engine::vigem_available() {
            "ok".to_string()
        } else {
            "not-responding".to_string()
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        "not-installed".to_string()
    }
}

#[tauri::command]
fn init_watcher(app: tauri::AppHandle, profile: Profile) -> Result<String, String> {
    input_engine::engine::init_watcher(profile)?;

    // Set up capture mode callback to emit events to frontend
    let app_handle = app.clone();
    input_engine::engine::set_capture_mode_callback(Box::new(move |active| {
        let _ = app_handle.emit("capture-mode-changed", active);
    }));

    // Forward engine diagnostics (hotkey registration, ViGEm status…) so the
    // UI can display them instead of failing silently
    let app_handle_logs = app.clone();
    input_engine::engine::set_log_callback(Box::new(move |msg| {
        let _ = app_handle_logs.emit("engine-log", msg.to_string());
    }));

    Ok("Watcher initialized".to_string())
}

/// Toggle capture mode from the UI (same effect as the global hotkey).
#[tauri::command]
fn toggle_capture() -> Result<(), String> {
    input_engine::engine::toggle_capture_mode();
    Ok(())
}

#[tauri::command]
fn shutdown_watcher() -> Result<String, String> {
    input_engine::engine::shutdown_watcher();
    Ok("Watcher shutdown".to_string())
}

#[tauri::command]
fn reload_profile(profile: Profile) -> Result<String, String> {
    input_engine::engine::reload_profile(profile)?;
    Ok("Profile reloaded".to_string())
}

/// Temporarily unregister the global capture hotkey (while the UI captures a
/// new key assignment, so the old hotkey cannot fire mid-assignment).
#[tauri::command]
fn suspend_hotkey() -> Result<(), String> {
    input_engine::engine::suspend_hotkey();
    Ok(())
}

/// Re-register the global capture hotkey from the current profile.
#[tauri::command]
fn resume_hotkey() -> Result<(), String> {
    input_engine::engine::resume_hotkey();
    Ok(())
}

#[tauri::command]
fn is_engine_running() -> bool {
    input_engine::engine::is_running()
}

#[tauri::command]
fn is_capture_mode_active() -> bool {
    input_engine::engine::is_capture_mode_active()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Restore the last saved window size, if any
            let app_handle = app.handle();
            if let Some(window) = app_handle.get_webview_window("main") {
                if let Some(state) = load_window_size(app_handle) {
                    *LAST_SAVED_SIZE.lock().unwrap() = Some(state);
                    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(
                        state.width,
                        state.height,
                    )));
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    // Emit a close-requested event to the frontend so it can show a confirmation dialog
                    // The frontend will call appWindow.close() again if the user confirms
                    let _ = window.emit("close-requested", ());
                    api.prevent_close();
                }
                tauri::WindowEvent::Resized(size) => {
                    // Never persist degenerate sizes or resizes from minimize/maximize
                    if size.width == 0 || size.height == 0 {
                        return;
                    }
                    if window.is_minimized().unwrap_or(false)
                        || window.is_maximized().unwrap_or(false)
                    {
                        return;
                    }
                    let new_size = WindowSizeState {
                        width: size.width,
                        height: size.height,
                    };
                    let mut last = LAST_SAVED_SIZE.lock().unwrap();
                    // Save only when the size actually differs from the last saved one
                    if *last != Some(new_size) {
                        *last = Some(new_size);
                        save_window_size(window.app_handle(), new_size);
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            check_vigembus,
            init_watcher,
            shutdown_watcher,
            reload_profile,
            is_engine_running,
            is_capture_mode_active,
            center_cursor,
            show_cursor,
            toggle_capture,
            suspend_hotkey,
            resume_hotkey,
            save_profile,
            load_profile,
            list_profiles,
            delete_profile,
            export_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
