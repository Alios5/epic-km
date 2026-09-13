//! Linux input capture via evdev — mirrors the Windows `win_capture` module.
//!
//! - Passively monitors every keyboard/pointer device under `/dev/input`
//!   (pressed-key set, mouse deltas, mouse buttons) so the mapper sees the
//!   same `RawInputState` as on Windows.
//! - The profile's toggle key flips capture mode (no OS hotkey registration
//!   needed — the key is detected directly in the evdev stream).
//! - While capture mode is active the devices are grabbed (`EVIOCGRAB`), so
//!   mapped keys and all mouse input stop reaching the desktop — the Linux
//!   equivalent of the Windows low-level hook + cursor clip. Unmapped
//!   keyboard keys are re-emitted through a uinput virtual keyboard so the
//!   rest of the system keeps working.
//!
//! Requires read (and grab) access to `/dev/input/event*` — typically via
//! the `input` group or a udev `uaccess` rule.

use super::*;
use crate::keycode::{code_to_scancode, is_mouse_code};
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, Device, EventType, InputEvent, Key, RelativeAxisType};
use std::os::unix::io::AsRawFd;
use std::sync::atomic::Ordering as AtomicOrdering;

// Global pointer to the shared state, set before the capture thread starts.
static mut SHARED_STATE: Option<Arc<EngineState>> = None;

static STOP: AtomicBool = AtomicBool::new(false);
static TOGGLE_REQUESTED: AtomicBool = AtomicBool::new(false);
static HOTKEY_SUSPENDED: AtomicBool = AtomicBool::new(false);
static REREGISTER_REQUESTED: AtomicBool = AtomicBool::new(false);

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

/// Ask the capture thread to stop (called from shutdown_watcher).
pub fn stop_capture() {
    STOP.store(true, AtomicOrdering::SeqCst);
}

/// Ask the capture thread to toggle capture mode (thread-safe).
pub fn post_toggle_message() {
    TOGGLE_REQUESTED.store(true, AtomicOrdering::SeqCst);
}

/// Re-read the toggle key from the profile and re-enable hotkey detection.
/// On Linux the toggle key is read live from the profile on every key event,
/// so "re-registering" only lifts the suspension and logs the resolved key.
pub fn post_reregister_message() {
    HOTKEY_SUSPENDED.store(false, AtomicOrdering::SeqCst);
    REREGISTER_REQUESTED.store(true, AtomicOrdering::SeqCst);
}

/// Suspend hotkey detection (e.g. while the UI captures a new key binding).
pub fn post_hotkey_off_message() {
    HOTKEY_SUSPENDED.store(true, AtomicOrdering::SeqCst);
}

/// Translate an evdev key code to the engine's layout-independent identifier
/// (set-1 make code, +0x100 for keys that are E0-extended on PC keyboards).
///
/// evdev codes 1..=88 are identical to set-1 make codes, so the whole main
/// cluster maps 1:1; only the extended/navigation block needs a table.
fn evdev_to_id(code: u16) -> u16 {
    match code {
        // Mouse buttons → pseudo-codes (see keycode.rs)
        0x110 => 0x200, // BTN_LEFT
        0x111 => 0x201, // BTN_RIGHT
        0x112 => 0x202, // BTN_MIDDLE
        0x113 => 0x203, // BTN_SIDE (X1)
        0x114 => 0x204, // BTN_EXTRA (X2)
        c if c < 89 => c,
        89 => 0x73,   // KEY_RO → IntlRo
        96 => 0x11C,  // KEY_KPENTER → NumpadEnter
        97 => 0x11D,  // KEY_RIGHTCTRL → ControlRight
        98 => 0x135,  // KEY_KPSLASH → NumpadDivide
        99 => 0x137,  // KEY_SYSRQ → PrintScreen
        100 => 0x138, // KEY_RIGHTALT → AltRight
        102 => 0x147, // KEY_HOME
        103 => 0x148, // KEY_UP
        104 => 0x149, // KEY_PAGEUP
        105 => 0x14B, // KEY_LEFT
        106 => 0x14D, // KEY_RIGHT
        107 => 0x14F, // KEY_END
        108 => 0x150, // KEY_DOWN
        109 => 0x151, // KEY_PAGEDOWN
        110 => 0x152, // KEY_INSERT
        111 => 0x153, // KEY_DELETE
        119 => 0x445, // KEY_PAUSE (E1 sequence)
        124 => 0x7D,  // KEY_YEN → IntlYen
        125 => 0x15B, // KEY_LEFTMETA
        126 => 0x15C, // KEY_RIGHTMETA
        127 => 0x15D, // KEY_COMPOSE → ContextMenu
        c @ 183..=194 => 0x64 + (c - 183), // KEY_F13..KEY_F24
        // Unknown high codes: keep them unique and out of every mapped range.
        c => c | 0x800,
    }
}

struct MonitoredDevice {
    dev: Device,
    is_pointer: bool,
    dead: bool,
}

/// Log the capture toggle key resolved from the current profile, so the
/// expected id can be compared with the "Key 0x... down" log lines.
fn log_toggle_key(state: &Arc<EngineState>) {
    let p = state.profile.lock();
    let id = code_to_scancode(&p.capture_toggle_key);
    if id == 0 {
        log(&format!(
            "Capture toggle key '{}' is not a known key — hotkey disabled",
            p.capture_toggle_key
        ));
    } else {
        log(&format!(
            "Capture toggle key: '{}' (id 0x{:03X})",
            p.capture_toggle_key, id
        ));
    }
}

/// Toggle capture mode — runs on the capture thread.
fn toggle_capture_state(state: &Arc<EngineState>) {
    let new_active = !state.capture_mode_active.load(Ordering::SeqCst);
    state.capture_mode_active.store(new_active, Ordering::SeqCst);
    if let Some(cb) = state.capture_mode_callback.lock().as_ref() {
        cb(new_active);
    }
    log(if new_active {
        "Capture mode ACTIVATED"
    } else {
        "Capture mode DEACTIVATED"
    });
}

/// Drop any inputs still marked as pressed so nothing gets stuck.
fn clear_input_state(state: &Arc<EngineState>) {
    let mut input = state.raw_input.lock();
    input.pressed_keys.clear();
    input.mouse_buttons = 0;
    input.smooth_rx = 0.0;
    input.smooth_ry = 0.0;
    input.smooth_lx = 0.0;
    input.smooth_ly = 0.0;
}

/// True if `id` is bound to a gamepad button or a left-stick direction in the
/// current profile — i.e. the key must be swallowed while capture is active.
fn is_mapped_key(state: &Arc<EngineState>, id: u16) -> bool {
    let profile = state.profile.lock();
    profile.keyboard_to_button.iter().any(|m| {
        let sc = code_to_scancode(&m.key);
        sc != 0 && !is_mouse_code(sc) && id == sc
    }) || profile.keyboard_to_left_stick.iter().any(|m| {
        let sc = code_to_scancode(&m.key);
        sc != 0 && id == sc
    })
}

/// Build a uinput virtual keyboard able to re-emit every key the grabbed
/// keyboards support, so unmapped keys keep working while capture is active.
fn build_forwarder(devices: &[MonitoredDevice]) -> Option<VirtualDevice> {
    let mut keys = AttributeSet::<Key>::new();
    for d in devices.iter().filter(|d| !d.dead) {
        if let Some(supported) = d.dev.supported_keys() {
            for key in supported.iter() {
                keys.insert(key);
            }
        }
    }
    let result = VirtualDeviceBuilder::new()
        .map(|b| b.name("Epic KM Keyboard Forward"))
        .and_then(|b| b.with_keys(&keys))
        .and_then(|b| b.build());
    match result {
        Ok(dev) => Some(dev),
        Err(e) => {
            log(&format!(
                "Could not create key-forwarding device: {} — unmapped keys will be swallowed while capture is active",
                e
            ));
            None
        }
    }
}

fn handle_event(
    state: &Arc<EngineState>,
    is_pointer: bool,
    grabbed: bool,
    forwarder: &mut Option<VirtualDevice>,
    ev: InputEvent,
) {
    match ev.event_type() {
        EventType::KEY => {
            let code = ev.code();
            let value = ev.value();
            let id = evdev_to_id(code);

            // Feed the input state from ANY device that reports keys —
            // macro pads and media-key event devices count too (Raw Input
            // on Windows likewise reports every keyboard-class device).
            if is_mouse_code(id) {
                // Mouse buttons → mouse_buttons bitmask (bits 0-4)
                if value != 2 {
                    let bit = (id - 0x200) as u8;
                    let mut input = state.raw_input.lock();
                    if value != 0 {
                        input.mouse_buttons |= 1 << bit;
                    } else {
                        input.mouse_buttons &= !(1 << bit);
                    }
                }
            } else {
                // Pressed-key set (repeat events leave it unchanged).
                let changed = {
                    let mut input = state.raw_input.lock();
                    if value == 0 {
                        input.pressed_keys.remove(&id)
                    } else {
                        input.pressed_keys.insert(id)
                    }
                };
                if changed && value != 2 {
                    log(&format!(
                        "Key 0x{:03X} {}",
                        id,
                        if value == 0 { "up" } else { "down" }
                    ));
                }
            }

            // Toggle key: detected in the stream, swallowed while grabbed
            // (RegisterHotKey consumes the key on Windows too).
            if value == 1 && !HOTKEY_SUSPENDED.load(AtomicOrdering::SeqCst) {
                let toggle_id = {
                    let p = state.profile.lock();
                    code_to_scancode(&p.capture_toggle_key)
                };
                if toggle_id != 0 && id == toggle_id {
                    toggle_capture_state(state);
                    return;
                }
            }

            // While grabbed, re-emit unmapped keys so the desktop keeps
            // working; mapped keys and mouse buttons are swallowed (like
            // the Windows hook + cursor clip).
            if grabbed && !is_mouse_code(id) && !is_mapped_key(state, id) {
                if let Some(f) = forwarder.as_mut() {
                    let _ = f.emit(&[InputEvent::new(EventType::KEY, code, value)]);
                }
            }
        }
        EventType::RELATIVE if is_pointer => {
            let mut input = state.raw_input.lock();
            match ev.code() {
                c if c == RelativeAxisType::REL_X.0 => input.mouse_dx += ev.value(),
                c if c == RelativeAxisType::REL_Y.0 => input.mouse_dy += ev.value(),
                _ => {}
            }
        }
        _ => {}
    }
}

pub fn capture_thread(state: Arc<EngineState>) {
    unsafe { SHARED_STATE = Some(state.clone()) };
    STOP.store(false, AtomicOrdering::SeqCst);
    TOGGLE_REQUESTED.store(false, AtomicOrdering::SeqCst);
    HOTKEY_SUSPENDED.store(false, AtomicOrdering::SeqCst);

    // Discover keyboards and pointer devices. Our own virtual devices are
    // skipped by name so we never read back what the engine emits.
    let mut devices: Vec<MonitoredDevice> = Vec::new();
    for (path, dev) in evdev::enumerate() {
        if let Some(name) = dev.name() {
            if name.starts_with("Epic KM") {
                continue;
            }
        }
        // Monitor every device that reports keys or pointer motion —
        // secondary key devices (media keys, macro pads) may carry the
        // toggle key even without a full alphanumeric set.
        let has_keys = dev
            .supported_keys()
            .map_or(false, |k| k.iter().next().is_some());
        let is_pointer = dev.supported_relative_axes().map_or(false, |a| {
            a.contains(RelativeAxisType::REL_X) && a.contains(RelativeAxisType::REL_Y)
        });
        if !has_keys && !is_pointer {
            continue;
        }
        unsafe {
            libc::fcntl(dev.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK);
        }
        log(&format!(
            "Monitoring {} ({})",
            path.display(),
            dev.name().unwrap_or("unnamed")
        ));
        devices.push(MonitoredDevice {
            dev,
            is_pointer,
            dead: false,
        });
    }
    if devices.is_empty() {
        log("No readable input devices under /dev/input — add your user to the 'input' group or install a udev uaccess rule");
    }
    log_toggle_key(&state);

    let mut grabbed = false;
    let mut forwarder: Option<VirtualDevice> = None;

    while !STOP.load(AtomicOrdering::SeqCst) && state.running.load(Ordering::SeqCst) {
        // Pending toggle requested from the UI thread.
        if TOGGLE_REQUESTED.swap(false, AtomicOrdering::SeqCst) {
            toggle_capture_state(&state);
        }
        // Profile reloaded — log the resolved toggle key for diagnostics.
        if REREGISTER_REQUESTED.swap(false, AtomicOrdering::SeqCst) {
            log_toggle_key(&state);
        }

        // Apply grab state transitions.
        let want_grab = state.capture_mode_active.load(Ordering::SeqCst);
        if want_grab != grabbed {
            if want_grab {
                let mut ok = 0usize;
                for d in devices.iter_mut().filter(|d| !d.dead) {
                    if d.dev.grab().is_ok() {
                        ok += 1;
                    }
                }
                forwarder = build_forwarder(&devices);
                log(&format!("Grabbed {} input device(s)", ok));
            } else {
                for d in devices.iter_mut().filter(|d| !d.dead) {
                    let _ = d.dev.ungrab();
                }
                forwarder = None;
                clear_input_state(&state);
            }
            grabbed = want_grab;
        }

        // Wait for input on any live device (20 ms tick also services the
        // atomic flags above).
        let mut poll_list: Vec<(usize, libc::pollfd)> = devices
            .iter()
            .enumerate()
            .filter(|(_, d)| !d.dead)
            .map(|(i, d)| {
                (
                    i,
                    libc::pollfd {
                        fd: d.dev.as_raw_fd(),
                        events: libc::POLLIN,
                        revents: 0,
                    },
                )
            })
            .collect();
        let n = unsafe {
            libc::poll(
                poll_list.as_mut_ptr() as *mut libc::pollfd,
                poll_list.len() as libc::nfds_t,
                20,
            )
        };
        if n <= 0 {
            continue;
        }

        for (idx, pfd) in &poll_list {
            if pfd.revents == 0 {
                continue;
            }
            let d = &mut devices[*idx];
            if pfd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
                d.dead = true;
                if grabbed {
                    let _ = d.dev.ungrab();
                }
                continue;
            }
            if pfd.revents & libc::POLLIN == 0 {
                continue;
            }
            let events: Vec<InputEvent> = match d.dev.fetch_events() {
                Ok(it) => it.collect(),
                Err(_) => continue,
            };
            let is_ptr = d.is_pointer;
            for ev in events {
                handle_event(&state, is_ptr, grabbed, &mut forwarder, ev);
            }
        }
    }

    // Cleanup: release every grab and drop pressed state.
    for d in devices.iter_mut() {
        let _ = d.dev.ungrab();
    }
    if state.capture_mode_active.swap(false, Ordering::SeqCst) {
        if let Some(cb) = state.capture_mode_callback.lock().as_ref() {
            cb(false);
        }
    }
    clear_input_state(&state);
    unsafe { SHARED_STATE = None };
    log("Capture thread stopped");
}
