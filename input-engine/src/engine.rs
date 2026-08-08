use crate::mapping::{map_input, GamepadState, RawInputState};
use crate::profile::Profile;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Callback type for capture mode changes.
pub type CaptureModeCallback = Box<dyn Fn(bool) + Send + Sync + 'static>;

/// Callback type for engine diagnostic messages (shown in the UI).
pub type LogCallback = Box<dyn Fn(&str) + Send + Sync + 'static>;

/// Shared engine state between capture and emission threads.
pub struct EngineState {
    pub raw_input: Mutex<RawInputState>,
    pub profile: Mutex<Profile>,
    pub running: AtomicBool,
    pub capture_mode_active: AtomicBool,
    pub capture_mode_callback: Mutex<Option<CaptureModeCallback>>,
    pub log_callback: Mutex<Option<LogCallback>>,
}

static mut ENGINE: Option<Arc<EngineState>> = None;
static mut CAPTURE_THREAD: Option<thread::JoinHandle<()>> = None;
static ENGINE_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());

fn get_engine() -> Option<&'static Arc<EngineState>> {
    unsafe { ENGINE.as_ref() }
}

/// Start the engine: plugs virtual gamepad, registers Raw Input + hotkey.
/// The gamepad is immediately active (emission thread running).
/// Capture mode (mouse lock + keyboard hook) is toggled by the hotkey.
pub fn init_watcher(profile: Profile) -> Result<(), String> {
    let lock = ENGINE_LOCK.lock();

    // If a previous engine is still running, shut it down first
    unsafe {
        if ENGINE.is_some() {
            drop(lock);
            shutdown_watcher();
            let lock2 = ENGINE_LOCK.lock();
            init_watcher_inner(profile, lock2)
        } else {
            init_watcher_inner(profile, lock)
        }
    }
}

fn init_watcher_inner(profile: Profile, _lock: parking_lot::MutexGuard<'_, ()>) -> Result<(), String> {
    unsafe {
        if ENGINE.is_some() {
            return Err("Engine is already running".to_string());
        }

        let state = Arc::new(EngineState {
            raw_input: Mutex::new(RawInputState::default()),
            profile: Mutex::new(profile),
            running: AtomicBool::new(true),
            capture_mode_active: AtomicBool::new(false),
            capture_mode_callback: Mutex::new(None),
            log_callback: Mutex::new(None),
        });

        ENGINE = Some(state.clone());

        // Start capture thread (raw input + hotkey + message loop)
        #[cfg(target_os = "windows")]
        {
            let state_clone = ENGINE.clone().unwrap();
            let handle = thread::spawn(move || {
                capture_thread(state_clone);
            });
            CAPTURE_THREAD = Some(handle);
        }

        // Start emission thread (ViGEmBus gamepad, always running)
        let state_clone = ENGINE.clone().unwrap();
        thread::spawn(move || {
            emission_thread(state_clone);
        });
    }

    Ok(())
}

/// Shutdown the watcher and stop the engine if running.
/// Blocks until the capture thread has fully terminated.
pub fn shutdown_watcher() {
    let lock = ENGINE_LOCK.lock();

    unsafe {
        if let Some(state) = ENGINE.take() {
            // Stop emission thread
            state.running.store(false, Ordering::SeqCst);

            // Deactivate capture mode if active
            if state.capture_mode_active.load(Ordering::SeqCst) {
                state.capture_mode_active.store(false, Ordering::SeqCst);
                #[cfg(target_os = "windows")]
                win_capture::deactivate_capture_mode();
                if let Some(cb) = state.capture_mode_callback.lock().as_ref() {
                    cb(false);
                }
            }

            // Stop capture thread
            #[cfg(target_os = "windows")]
            win_capture::stop_capture();
        }

        // Wait for capture thread to fully terminate
        #[cfg(target_os = "windows")]
        if let Some(handle) = CAPTURE_THREAD.take() {
            drop(lock);
            let _ = handle.join();
            return;
        }
    }

    drop(lock);
}

/// Reload the profile without restarting threads.
pub fn reload_profile(profile: Profile) -> Result<(), String> {
    unsafe {
        if let Some(state) = ENGINE.as_ref() {
            let mut p = state.profile.lock();
            *p = profile;
            drop(p);
            // The capture thread owns the hotkey registration — ask it to
            // re-register in case the toggle key changed.
            #[cfg(target_os = "windows")]
            win_capture::post_reregister_message();
            Ok(())
        } else {
            Err("Watcher is not running".to_string())
        }
    }
}

/// Temporarily unregister the global capture hotkey (e.g. while the user is
/// assigning a new key in the UI, so pressing the old key does not toggle
/// capture mid-assignment).
pub fn suspend_hotkey() {
    #[cfg(target_os = "windows")]
    win_capture::post_hotkey_off_message();
}

/// Re-register the global capture hotkey from the current profile.
pub fn resume_hotkey() {
    #[cfg(target_os = "windows")]
    win_capture::post_reregister_message();
}

/// Check if the engine is running.
pub fn is_running() -> bool {
    unsafe {
        ENGINE.as_ref().map_or(false, |s| s.running.load(Ordering::SeqCst))
    }
}

/// Set a callback that fires when capture mode changes.
pub fn set_capture_mode_callback(callback: CaptureModeCallback) {
    unsafe {
        if let Some(state) = ENGINE.as_ref() {
            let mut cb = state.capture_mode_callback.lock();
            *cb = Some(callback);
        }
    }
}

/// Set a callback that receives engine diagnostic messages.
pub fn set_log_callback(callback: LogCallback) {
    unsafe {
        if let Some(state) = ENGINE.as_ref() {
            let mut cb = state.log_callback.lock();
            *cb = Some(callback);
        }
    }
}

/// Toggle capture mode programmatically (same effect as the global hotkey).
/// The work happens on the capture thread, which owns the OS hooks.
pub fn toggle_capture_mode() {
    #[cfg(target_os = "windows")]
    win_capture::post_toggle_message();
}

/// Log a message to stdout and forward it to the UI log callback (if any).
fn elog(state: &EngineState, msg: &str) {
    println!("[input-engine] {}", msg);
    if let Some(cb) = state.log_callback.lock().as_ref() {
        cb(msg);
    }
}

/// Check if capture mode is currently active.
pub fn is_capture_mode_active() -> bool {
    unsafe {
        ENGINE.as_ref().map_or(false, |s| s.capture_mode_active.load(Ordering::SeqCst))
    }
}

/// Quick functional check: is the ViGEmBus driver present and accepting
/// clients? Opens a bus connection and immediately drops it.
pub fn vigem_available() -> bool {
    #[cfg(target_os = "windows")]
    {
        vigem_client::Client::connect().is_ok()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// Emission thread: reads shared state at fixed frequency, applies mapping,
/// sends to ViGEmBus virtual gamepad.
fn emission_thread(state: Arc<EngineState>) {
    #[cfg(target_os = "windows")]
    {
        use vigem_client::{Client, TargetId, XButtons, XGamepad, Xbox360Wired};
        use windows::Win32::Media::{timeBeginPeriod, timeEndPeriod};
        use windows::Win32::System::Threading::{
            GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_HIGHEST,
        };

        let client = match Client::connect() {
            Ok(c) => c,
            Err(e) => {
                elog(&state, &format!("Failed to connect to ViGEmBus: {}", e));
                state.running.store(false, Ordering::SeqCst);
                return;
            }
        };

        let mut target = Xbox360Wired::new(client, TargetId::XBOX360_WIRED);
        if let Err(e) = target.plugin() {
            elog(&state, &format!("Failed to plugin ViGEmBus target: {}", e));
            state.running.store(false, Ordering::SeqCst);
            return;
        }

        // Wait for target to be ready
        if let Err(e) = target.wait_ready() {
            elog(&state, &format!("ViGEmBus target not ready: {}", e));
            state.running.store(false, Ordering::SeqCst);
            return;
        }

        // Low-latency setup: 1 ms timer resolution, a high-priority thread to
        // minimise scheduling jitter, and a HIGH-RESOLUTION waitable timer
        // (Win10 1803+) for sub-ms pacing without busy-spinning.
        unsafe {
            timeBeginPeriod(1);
            let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);
        }

        use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
        use windows::Win32::System::Threading::{
            CreateWaitableTimerExW, SetWaitableTimerEx, WaitForSingleObject,
            CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, TIMER_ALL_ACCESS,
        };

        /// Current QPC time in 100 ns units (overflow-safe).
        fn qpc_100ns(freq: i64) -> i64 {
            unsafe {
                let mut c = 0i64;
                let _ = QueryPerformanceCounter(&mut c);
                (c / freq) * 10_000_000 + (c % freq) * 10_000_000 / freq
            }
        }

        let qpc_freq = unsafe {
            let mut f = 0i64;
            let _ = QueryPerformanceFrequency(&mut f);
            f
        };
        let hi_res_timer = unsafe {
            CreateWaitableTimerExW(
                None,
                windows::core::PCWSTR::null(),
                CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                TIMER_ALL_ACCESS.0,
            )
            .ok()
        };
        let mut next_tick = qpc_100ns(qpc_freq);

        elog(&state, "Emission thread started — ViGEmBus connected");

        while state.running.load(Ordering::SeqCst) {
            // Read polling rate from profile (period in 100 ns units)
            let hz = {
                let p = state.profile.lock();
                p.right_stick.refresh_interval.max(1)
            };
            let period_100ns = 10_000_000i64 / hz as i64;

            // Map input — the whole gamepad report is gated on capture mode.
            // When capture is OFF (mouse2joystick-style idle state), the virtual
            // gamepad must stay fully neutral: no mouse, no keyboard, no buttons.
            let capture_active = state.capture_mode_active.load(Ordering::SeqCst);
            let gamepad_state = {
                let mut raw = state.raw_input.lock();
                let profile = state.profile.lock();
                if capture_active {
                    map_input(&mut raw, &profile)
                } else {
                    // Drain accumulated mouse deltas so they don't leak into the
                    // first report when capture mode is re-activated.
                    raw.mouse_dx = 0;
                    raw.mouse_dy = 0;
                    GamepadState::default()
                }
            };

            // Center cursor when capture mode is active
            #[cfg(target_os = "windows")]
            if capture_active {
                use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SetCursorPos, SM_CXSCREEN, SM_CYSCREEN};
                unsafe {
                    let cx = GetSystemMetrics(SM_CXSCREEN);
                    let cy = GetSystemMetrics(SM_CYSCREEN);
                    let _ = SetCursorPos(cx / 2, cy / 2);
                }
            }

            // Build XUSB report
            let b = gamepad_state.buttons;
            let mut raw_buttons: u16 = 0;
            if b.dpad_up { raw_buttons |= XButtons::UP; }
            if b.dpad_down { raw_buttons |= XButtons::DOWN; }
            if b.dpad_left { raw_buttons |= XButtons::LEFT; }
            if b.dpad_right { raw_buttons |= XButtons::RIGHT; }
            if b.start { raw_buttons |= XButtons::START; }
            if b.back { raw_buttons |= XButtons::BACK; }
            if b.left_thumb { raw_buttons |= XButtons::LTHUMB; }
            if b.right_thumb { raw_buttons |= XButtons::RTHUMB; }
            if b.left_shoulder { raw_buttons |= XButtons::LB; }
            if b.right_shoulder { raw_buttons |= XButtons::RB; }
            if b.a { raw_buttons |= XButtons::A; }
            if b.b { raw_buttons |= XButtons::B; }
            if b.x { raw_buttons |= XButtons::X; }
            if b.y { raw_buttons |= XButtons::Y; }

            let report = XGamepad {
                buttons: XButtons { raw: raw_buttons },
                left_trigger: if b.left_trigger { 255 } else { 0 },
                right_trigger: if b.right_trigger { 255 } else { 0 },
                thumb_lx: gamepad_state.left_stick_x,
                thumb_ly: gamepad_state.left_stick_y,
                thumb_rx: gamepad_state.right_stick_x,
                thumb_ry: gamepad_state.right_stick_y,
            };

            if let Err(e) = target.update(&report) {
                eprintln!("[input-engine] ViGEmBus update error: {}", e);
            }

            // Absolute-deadline pacing (no drift): wait until the next tick.
            next_tick += period_100ns;
            if let Some(timer) = hi_res_timer {
                let wait = next_tick - qpc_100ns(qpc_freq);
                if wait > 0 {
                    let due = -wait; // negative = relative time, 100 ns units
                    unsafe {
                        let _ = SetWaitableTimerEx(timer, &due, 0, None, None, None, 0);
                        let _ = WaitForSingleObject(timer, 0xFFFFFFFF); // INFINITE
                    }
                }
            } else {
                // Fallback: sleep the bulk, spin the last ~1 ms.
                loop {
                    let remaining = next_tick - qpc_100ns(qpc_freq);
                    if remaining <= 0 { break; }
                    if remaining > 15_000 {
                        thread::sleep(Duration::from_micros(((remaining - 10_000) / 10).max(1) as u64));
                    } else {
                        std::hint::spin_loop();
                    }
                }
            }
            // Resync after a system stall so we don't emit catch-up bursts.
            let now = qpc_100ns(qpc_freq);
            if now - next_tick > period_100ns { next_tick = now; }
        }

        if let Some(t) = hi_res_timer {
            unsafe { let _ = windows::Win32::Foundation::CloseHandle(t); };
        }
        unsafe { timeEndPeriod(1); }
        let _ = target.unplug();
        elog(&state, "Emission thread stopped");
    }

    #[cfg(not(target_os = "windows"))]
    {
        while state.running.load(Ordering::SeqCst) {
            let hz = {
                let p = state.profile.lock();
                p.right_stick.refresh_interval.max(1)
            };
            let interval_us = 1_000_000 / hz as u64;
            thread::sleep(Duration::from_micros(interval_us));
        }
    }
}

// ---- Platform-specific capture integration ----

#[cfg(target_os = "windows")]
mod win_capture {
    use super::*;
    use crate::keycode::{code_to_scancode, is_mouse_code};
    use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
    use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::Input::{
        GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE,
        RAWINPUTHEADER, RID_INPUT, RIM_TYPEKEYBOARD, RIM_TYPEMOUSE,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, GetMessageW, PostThreadMessageW,
        RegisterClassW, TranslateMessage, DispatchMessageW,
        WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSW, WM_INPUT, WM_QUIT, HWND_MESSAGE,
        CS_HREDRAW, CS_VREDRAW, RI_KEY_BREAK, RI_KEY_E0, RI_KEY_E1,
        RI_MOUSE_LEFT_BUTTON_DOWN, RI_MOUSE_LEFT_BUTTON_UP,
        RI_MOUSE_RIGHT_BUTTON_DOWN, RI_MOUSE_RIGHT_BUTTON_UP,
        RI_MOUSE_MIDDLE_BUTTON_DOWN, RI_MOUSE_MIDDLE_BUTTON_UP,
        WM_HOTKEY, WM_APP, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
        SetWindowsHookExW, UnhookWindowsHookEx, CallNextHookEx,
        HHOOK, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, HC_ACTION, LLKHF_EXTENDED,
        ShowCursor, ClipCursor, GetSystemMetrics, SetCursorPos,
        SM_CXSCREEN, SM_CYSCREEN,
        GetForegroundWindow, GetWindowThreadProcessId,
    };
    use windows::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        RegisterHotKey, UnregisterHotKey, MOD_NOREPEAT,
        GetKeyboardLayout, MapVirtualKeyExW, MAPVK_VSC_TO_VK_EX,
    };
    use windows::core::w;

    static mut THREAD_ID: u32 = 0;
    static CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(false);
    const HOTKEY_ID: i32 = 0xB001;
    // Custom thread messages handled by the capture thread's message loop
    const WM_APP_TOGGLE: u32 = WM_APP + 1;
    const WM_APP_REREGISTER: u32 = WM_APP + 2;
    const WM_APP_HOTKEY_OFF: u32 = WM_APP + 3;

    // Global pointer to the shared state, set before the capture thread starts.
    static mut SHARED_STATE: Option<Arc<EngineState>> = None;

    // Low-level keyboard hook handle
    static mut KB_HOOK: Option<HHOOK> = None;

    pub fn set_shared_state(state: Arc<EngineState>) {
        unsafe { SHARED_STATE = Some(state); }
    }

    pub fn stop_capture() {
        unsafe {
            if THREAD_ID != 0 {
                let _ = PostThreadMessageW(THREAD_ID, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }
        CAPTURE_ACTIVE.store(false, AtomicOrdering::SeqCst);
    }

    /// Log to stdout and forward to the UI log callback (if any).
    fn log(msg: &str) {
        println!("[input-engine] {}", msg);
        unsafe {
            if let Some(state) = SHARED_STATE.as_ref() {
                if let Some(cb) = state.log_callback.lock().as_ref() {
                    cb(msg);
                }
            }
        }
    }

    /// Ask the capture thread to toggle capture mode (thread-safe).
    pub fn post_toggle_message() {
        unsafe {
            if THREAD_ID != 0 {
                let _ = PostThreadMessageW(THREAD_ID, WM_APP_TOGGLE, WPARAM(0), LPARAM(0));
            }
        }
    }

    /// Ask the capture thread to re-register the hotkey from the profile.
    pub fn post_reregister_message() {
        unsafe {
            if THREAD_ID != 0 {
                let _ = PostThreadMessageW(THREAD_ID, WM_APP_REREGISTER, WPARAM(0), LPARAM(0));
            }
        }
    }

    /// Ask the capture thread to unregister the hotkey (suspend).
    pub fn post_hotkey_off_message() {
        unsafe {
            if THREAD_ID != 0 {
                let _ = PostThreadMessageW(THREAD_ID, WM_APP_HOTKEY_OFF, WPARAM(0), LPARAM(0));
            }
        }
    }

    /// Toggle capture mode — shared by WM_HOTKEY and WM_APP_TOGGLE.
    fn toggle_capture_state() {
        unsafe {
            if let Some(state) = SHARED_STATE.as_ref() {
                let new_active = !state.capture_mode_active.load(Ordering::SeqCst);
                state.capture_mode_active.store(new_active, Ordering::SeqCst);
                if new_active {
                    activate_capture_mode();
                } else {
                    deactivate_capture_mode();
                }
                if let Some(cb) = state.capture_mode_callback.lock().as_ref() {
                    cb(new_active);
                }
                log(if new_active { "Capture mode ACTIVATED" } else { "Capture mode DEACTIVATED" });
            }
        }
    }

    /// Unregister the global hotkey (suspend) — used while the UI captures a
    /// new key assignment so the old hotkey cannot fire mid-assignment.
    fn unregister_hotkey(hwnd: HWND) {
        unsafe {
            let _ = UnregisterHotKey(Some(hwnd), HOTKEY_ID);
            log("Global hotkey temporarily unregistered");
        }
    }

    /// (Re)register the global hotkey from the current profile.
    /// Must run on the capture thread, which owns the window.
    fn reregister_hotkey(hwnd: HWND) {
        unsafe {
            let _ = UnregisterHotKey(Some(hwnd), HOTKEY_ID);
            if let Some(state) = SHARED_STATE.as_ref() {
                let vk = {
                    let p = state.profile.lock();
                    let id = code_to_scancode(&p.capture_toggle_key);
                    if id == 0 || is_mouse_code(id) {
                        0
                    } else {
                        scancode_to_vk(id)
                    }
                };
                if vk == 0 {
                    log("Hotkey unregistered (no valid toggle key in profile)");
                    return;
                }
                match RegisterHotKey(Some(hwnd), HOTKEY_ID, MOD_NOREPEAT, vk) {
                    Ok(_) => log(&format!("Global hotkey re-registered (VK=0x{:02X})", vk)),
                    Err(_) => {
                        let err = windows::Win32::Foundation::GetLastError().0;
                        log(&format!("RegisterHotKey failed: {} (VK=0x{:02X})", err, vk));
                    }
                }
            }
        }
    }

    extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_INPUT {
            handle_raw_input(lparam);
            return LRESULT(0);
        }
        if msg == WM_HOTKEY || msg == WM_APP_TOGGLE {
            toggle_capture_state();
            return LRESULT(0);
        }
        if msg == WM_APP_REREGISTER {
            reregister_hotkey(hwnd);
            return LRESULT(0);
        }
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    /// Activate capture mode: hide cursor, clip cursor, install keyboard hook.
    pub fn activate_capture_mode() {
        unsafe {
            // Hide cursor
            while ShowCursor(false) >= 0 {}

            // Clip cursor to center point (1x1 pixel rect)
            let cx = GetSystemMetrics(SM_CXSCREEN);
            let cy = GetSystemMetrics(SM_CYSCREEN);
            let rect = windows::Win32::Foundation::RECT {
                left: cx / 2,
                top: cy / 2,
                right: cx / 2 + 1,
                bottom: cy / 2 + 1,
            };
            let _ = ClipCursor(Some(&rect));

            // Center cursor
            let _ = SetCursorPos(cx / 2, cy / 2);

            // Install low-level keyboard hook
            let hinstance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
                .expect("GetModuleHandleW failed");
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(kb_hook_proc), Some(HINSTANCE(hinstance.0)), 0);
            if let Ok(h) = hook {
                KB_HOOK = Some(h);
                log("Keyboard hook installed");
            } else {
                log("Failed to install keyboard hook — mapped keys will NOT be blocked");
            }
        }
    }

    /// Deactivate capture mode: restore cursor, unclip, remove keyboard hook.
    pub fn deactivate_capture_mode() {
        unsafe {
            // Remove keyboard hook
            if let Some(h) = KB_HOOK.take() {
                let _ = UnhookWindowsHookEx(h);
            }

            // Drop any inputs still marked as pressed so nothing gets stuck
            if let Some(state) = SHARED_STATE.as_ref() {
                let mut input = state.raw_input.lock();
                input.pressed_keys.clear();
                input.mouse_buttons = 0;
                input.smooth_rx = 0.0;
                input.smooth_ry = 0.0;
                input.smooth_lx = 0.0;
                input.smooth_ly = 0.0;
            }

            // Unclip cursor
            let _ = ClipCursor(None);

            // Show cursor
            while ShowCursor(true) < 0 {}
        }
    }

    /// Convert a scan code identifier (see keycode.rs) to the virtual key code
    /// of the *current* keyboard layout — needed for RegisterHotKey, which only
    /// accepts virtual key codes.
    fn scancode_to_vk(id: u16) -> u32 {
        let make = (id & 0xFF) as u32;
        let extended = (id & 0x100) != 0;
        // Extended keys are passed with 0xE0 in the high byte of the scan code
        let uscan_code = make | if extended { 0xE000 } else { 0 };
        unsafe { MapVirtualKeyExW(uscan_code, MAPVK_VSC_TO_VK_EX, Some(GetKeyboardLayout(0))) }
    }

    /// True when the foreground window belongs to this process (our own UI).
    /// Used to avoid swallowing the user's keystrokes while they edit mappings.
    fn foreground_is_our_window() -> bool {
        unsafe {
            let fg = GetForegroundWindow();
            if fg.0.is_null() {
                return false;
            }
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(fg, Some(&mut pid));
            pid != 0 && pid == GetCurrentProcessId()
        }
    }

    /// Low-level keyboard hook: blocks mapped keys from propagating.
    extern "system" fn kb_hook_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if code as u32 == HC_ACTION {
            let kb = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
            // Layout-independent identifier: scan code + extended flag
            let mut id = kb.scanCode as u16;
            if (kb.flags & LLKHF_EXTENDED).0 != 0 {
                id |= 0x100;
            }

            // Feed the pressed-key set. While capture mode is active the hook
            // is the authoritative keyboard source: it provably sees every key
            // (it has to, in order to block them), whereas Raw Input keyboard
            // messages become unreliable once the hook swallows a key.
            let is_down = {
                let msg = wparam.0 as u32;
                msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN
            };
            unsafe {
                if let Some(state) = SHARED_STATE.as_ref() {
                    let changed = {
                        let mut input = state.raw_input.lock();
                        if is_down {
                            input.pressed_keys.insert(id)
                        } else {
                            input.pressed_keys.remove(&id)
                        }
                    };
                    if changed {
                        log(&format!("Key 0x{:03X} {} (hook)", id, if is_down { "down" } else { "up" }));
                    }
                }
            }

            // Check if this key is mapped in the profile
            let should_block = unsafe {
                SHARED_STATE.as_ref().map_or(false, |state| {
                    let capture_active = state.capture_mode_active.load(Ordering::SeqCst);
                    if !capture_active {
                        return false;
                    }
                    // Never swallow keys while our own window is in the foreground,
                    // so mappings stay editable even with capture mode ON.
                    if foreground_is_our_window() {
                        return false;
                    }
                    let profile = state.profile.lock();
                    // Block the toggle key itself? No — let it through so WM_HOTKEY fires.
                    let toggle_id = code_to_scancode(&profile.capture_toggle_key);
                    if toggle_id != 0 && id == toggle_id {
                        return false;
                    }
                    // Block keys mapped to buttons or to the left stick
                    let blocks_button = profile.keyboard_to_button.iter().any(|m| {
                        let sc = code_to_scancode(&m.key);
                        sc != 0 && !is_mouse_code(sc) && id == sc
                    });
                    blocks_button || profile.keyboard_to_left_stick.iter().any(|m| {
                        let sc = code_to_scancode(&m.key);
                        sc != 0 && id == sc
                    })
                })
            };

            if should_block {
                // Return non-zero to block the key from propagating
                return LRESULT(1);
            }
        }
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }

    fn handle_raw_input(lparam: LPARAM) {
        unsafe {
            let state = match SHARED_STATE.as_ref() {
                Some(s) => s.clone(),
                None => return,
            };

            let mut raw: RAWINPUT = std::mem::zeroed();
            let mut size = std::mem::size_of::<RAWINPUT>() as u32;
            let copied = GetRawInputData(
                HRAWINPUT(lparam.0 as *mut _),
                RID_INPUT,
                Some(&mut raw as *mut _ as *mut core::ffi::c_void),
                &mut size as *mut u32,
                std::mem::size_of::<RAWINPUTHEADER>() as u32,
            );
            if copied == 0 {
                return;
            }

            let dw_type = raw.header.dwType;
            let mut input = state.raw_input.lock();

            if dw_type == RIM_TYPEKEYBOARD.0 {
                let kb = raw.data.keyboard;
                let make = kb.MakeCode;
                let flags = kb.Flags as u32;
                // Ignore "fake shift" messages Windows emits around extended keys
                if make == 0 || (make == 0x2A && (flags & RI_KEY_E0) != 0) {
                    return;
                }
                // Layout-independent identifier: make code + extension flags
                let mut id = make;
                if (flags & RI_KEY_E0) != 0 {
                    id |= 0x100;
                }
                if (flags & RI_KEY_E1) != 0 {
                    id |= 0x400;
                }
                // RI_KEY_MAKE is 0: everything that is not a BREAK is a press
                let changed = if (flags & RI_KEY_BREAK) != 0 {
                    input.pressed_keys.remove(&id)
                } else {
                    input.pressed_keys.insert(id)
                };
                if changed {
                    log(&format!("Key 0x{:03X} {} (raw)", id, if (flags & RI_KEY_BREAK) != 0 { "up" } else { "down" }));
                }
            } else if dw_type == RIM_TYPEMOUSE.0 {
                let mouse = raw.data.mouse;
                let dx: i32 = mouse.lLastX;
                let dy: i32 = mouse.lLastY;
                let button_flags = mouse.Anonymous.Anonymous.usButtonFlags as u32;

                // Accumulate deltas
                input.mouse_dx += dx;
                input.mouse_dy += dy;

                // Handle button presses
                if (button_flags & RI_MOUSE_LEFT_BUTTON_DOWN) != 0 {
                    input.mouse_buttons |= 1 << 0;
                }
                if (button_flags & RI_MOUSE_LEFT_BUTTON_UP) != 0 {
                    input.mouse_buttons &= !(1 << 0);
                }
                if (button_flags & RI_MOUSE_RIGHT_BUTTON_DOWN) != 0 {
                    input.mouse_buttons |= 1 << 1;
                }
                if (button_flags & RI_MOUSE_RIGHT_BUTTON_UP) != 0 {
                    input.mouse_buttons &= !(1 << 1);
                }
                if (button_flags & RI_MOUSE_MIDDLE_BUTTON_DOWN) != 0 {
                    input.mouse_buttons |= 1 << 2;
                }
                if (button_flags & RI_MOUSE_MIDDLE_BUTTON_UP) != 0 {
                    input.mouse_buttons &= !(1 << 2);
                }
            }
        }
    }

    pub fn capture_thread(state: Arc<EngineState>) {
        set_shared_state(state);

        unsafe {
            THREAD_ID = GetCurrentThreadId();

            let hinstance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
                .expect("GetModuleHandleW failed");

            let class_name = w!("InputEngineRawInputClass");

            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: HINSTANCE(hinstance.0),
                lpszClassName: class_name,
                ..std::mem::zeroed()
            };

            let atom = RegisterClassW(&wnd_class);
            if atom == 0 {
                let err = windows::Win32::Foundation::GetLastError().0;
                // ERROR_CLASS_ALREADY_EXISTS (1410): the window class stays
                // registered for the whole process lifetime — reuse it when
                // the engine restarts (e.g. re-entering the editor screen).
                if err != 1410 {
                    log(&format!("RegisterClassW failed: {}", err));
                    return;
                }
            }

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                w!("InputEngine"),
                WINDOW_STYLE::default(),
                0, 0, 0, 0,
                Some(HWND_MESSAGE),
                None,
                Some(HINSTANCE(hinstance.0)),
                None,
            );

            let hwnd = match hwnd {
                Ok(h) if !h.0.is_null() => h,
                _ => {
                    let err = windows::Win32::Foundation::GetLastError().0;
                    log(&format!("CreateWindowExW failed: {}", err));
                    return;
                }
            };

            // Register Raw Input devices at startup (keyboard + mouse)
            {
                use windows::Win32::UI::Input::RIDEV_INPUTSINK;
                let devices = [
                    RAWINPUTDEVICE {
                        usUsagePage: 0x01,
                        usUsage: 0x06,
                        dwFlags: RIDEV_INPUTSINK,
                        hwndTarget: hwnd,
                    },
                    RAWINPUTDEVICE {
                        usUsagePage: 0x01,
                        usUsage: 0x02,
                        dwFlags: RIDEV_INPUTSINK,
                        hwndTarget: hwnd,
                    },
                ];
                let result = RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32);
                if result.is_err() {
                    let err = windows::Win32::Foundation::GetLastError().0;
                    log(&format!("RegisterRawInputDevices failed: {}", err));
                } else {
                    log("Raw Input registered");
                }
            }

            // Register global hotkey for capture toggle. RegisterHotKey only
            // accepts a virtual key code: convert the (layout-independent) scan
            // code through the current keyboard layout.
            let toggle_key = {
                let p = SHARED_STATE.as_ref().unwrap().profile.lock();
                let id = code_to_scancode(&p.capture_toggle_key);
                if id == 0 || is_mouse_code(id) {
                    0
                } else {
                    scancode_to_vk(id)
                }
            };
            if toggle_key != 0 {
                // MOD_NOREPEAT: holding the key must not machine-gun toggles.
                // Retry registration — Windows may take time to release a hotkey
                // from a recently-terminated process.
                let mut registered = false;
                for attempt in 1..=10u32 {
                    let _ = UnregisterHotKey(Some(hwnd), HOTKEY_ID);
                    let result = RegisterHotKey(Some(hwnd), HOTKEY_ID, MOD_NOREPEAT, toggle_key);
                    if result.is_ok() {
                        log(&format!("Global hotkey registered (VK=0x{:02X}) on attempt {}", toggle_key, attempt));
                        registered = true;
                        break;
                    }
                    let err = windows::Win32::Foundation::GetLastError().0;
                    if err != 1409 {
                        // Not ERROR_HOTKEY_ALREADY_REGISTERED — log and stop retrying
                        log(&format!("RegisterHotKey failed: {} (VK=0x{:02X})", err, toggle_key));
                        break;
                    }
                    thread::sleep(Duration::from_millis(500));
                }
                if !registered {
                    log("FAILED to register hotkey after 10 attempts — another app may already own this key. Try changing the capture toggle key.");
                }
            } else {
                log("No valid capture toggle key in profile — hotkey NOT registered");
            }

            CAPTURE_ACTIVE.store(true, AtomicOrdering::SeqCst);
            log("Capture thread started — press the hotkey to toggle capture");

            let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                // Thread messages posted via PostThreadMessageW have a NULL
                // hwnd — DispatchMessageW silently drops them (no window
                // procedure to call), so they must be handled here directly.
                if msg.message == WM_APP_TOGGLE {
                    toggle_capture_state();
                    continue;
                }
                if msg.message == WM_APP_REREGISTER {
                    reregister_hotkey(hwnd);
                    continue;
                }
                if msg.message == WM_APP_HOTKEY_OFF {
                    unregister_hotkey(hwnd);
                    continue;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // Cleanup: deactivate capture mode if active
            let state_ref = SHARED_STATE.as_ref().unwrap();
            if state_ref.capture_mode_active.load(Ordering::SeqCst) {
                deactivate_capture_mode();
                state_ref.capture_mode_active.store(false, Ordering::SeqCst);
            }

            // Unregister raw input
            {
                use windows::Win32::UI::Input::RIDEV_REMOVE;
                let devices = [
                    RAWINPUTDEVICE {
                        usUsagePage: 0x01,
                        usUsage: 0x06,
                        dwFlags: RIDEV_REMOVE,
                        hwndTarget: HWND::default(),
                    },
                    RAWINPUTDEVICE {
                        usUsagePage: 0x01,
                        usUsage: 0x02,
                        dwFlags: RIDEV_REMOVE,
                        hwndTarget: HWND::default(),
                    },
                ];
                let _ = RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32);
            }

            // Unregister hotkey
            let _ = UnregisterHotKey(Some(hwnd), HOTKEY_ID);

            let _ = DestroyWindow(hwnd);
            log("Capture thread stopped");
            THREAD_ID = 0;
        }

        CAPTURE_ACTIVE.store(false, AtomicOrdering::SeqCst);
        unsafe { SHARED_STATE = None; }
    }
}

/// Entry point for the capture thread (called from init_watcher).
#[cfg(target_os = "windows")]
fn capture_thread(state: Arc<EngineState>) {
    win_capture::capture_thread(state);
}

#[cfg(not(target_os = "windows"))]
fn capture_thread(_state: Arc<EngineState>) {
    // No-op on non-Windows
}
