use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::{
    GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE,
    RAWINPUTHEADER, RID_INPUT, RIM_TYPEKEYBOARD, RIM_TYPEMOUSE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetMessageW, PostThreadMessageW,
    RegisterClassW, TranslateMessage, DispatchMessageW,
    WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSW, WM_INPUT, WM_QUIT, HWND_MESSAGE,
    CS_HREDRAW, CS_VREDRAW, RI_KEY_MAKE, RI_KEY_BREAK,
    RI_MOUSE_LEFT_BUTTON_DOWN, RI_MOUSE_LEFT_BUTTON_UP,
    RI_MOUSE_RIGHT_BUTTON_DOWN, RI_MOUSE_RIGHT_BUTTON_UP,
    RI_MOUSE_MIDDLE_BUTTON_DOWN, RI_MOUSE_MIDDLE_BUTTON_UP,
    RI_MOUSE_WHEEL,
};
use windows::core::w;

static CAPTURING: AtomicBool = AtomicBool::new(false);
static mut THREAD_ID: u32 = 0;

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
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn handle_raw_input(lparam: LPARAM) {
    unsafe {
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
        if dw_type == RIM_TYPEKEYBOARD.0 {
            let kb = raw.data.keyboard;
            let vkey = kb.VKey;
            let flags = kb.Flags as u32;
            if (flags & RI_KEY_BREAK) != 0 {
                println!("[input-engine] Key release: VK=0x{:02X} ({})", vkey, vkey);
            } else if (flags & RI_KEY_MAKE) == 0 {
                println!("[input-engine] Key press:   VK=0x{:02X} ({})", vkey, vkey);
            }
        } else if dw_type == RIM_TYPEMOUSE.0 {
            let mouse = raw.data.mouse;
            let dx: i32 = mouse.lLastX;
            let dy: i32 = mouse.lLastY;
            let button_flags = mouse.Anonymous.Anonymous.usButtonFlags as u32;

            if (button_flags & RI_MOUSE_LEFT_BUTTON_DOWN) != 0 {
                println!("[input-engine] Mouse LEFT button DOWN");
            }
            if (button_flags & RI_MOUSE_LEFT_BUTTON_UP) != 0 {
                println!("[input-engine] Mouse LEFT button UP");
            }
            if (button_flags & RI_MOUSE_RIGHT_BUTTON_DOWN) != 0 {
                println!("[input-engine] Mouse RIGHT button DOWN");
            }
            if (button_flags & RI_MOUSE_RIGHT_BUTTON_UP) != 0 {
                println!("[input-engine] Mouse RIGHT button UP");
            }
            if (button_flags & RI_MOUSE_MIDDLE_BUTTON_DOWN) != 0 {
                println!("[input-engine] Mouse MIDDLE button DOWN");
            }
            if (button_flags & RI_MOUSE_MIDDLE_BUTTON_UP) != 0 {
                println!("[input-engine] Mouse MIDDLE button UP");
            }
            if (button_flags & RI_MOUSE_WHEEL) != 0 {
                let wheel_delta = mouse.Anonymous.Anonymous.usButtonData as i16;
                println!("[input-engine] Mouse wheel: delta={}", wheel_delta);
            }
            if button_flags == 0 {
                println!("[input-engine] Mouse move: dx={}, dy={}", dx, dy);
            }
        }
    }
}

fn capture_thread() {
    unsafe {
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
            println!("[input-engine] RegisterClassW failed: {}",
                windows::Win32::Foundation::GetLastError().0);
            CAPTURING.store(false, Ordering::SeqCst);
            return;
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
                println!("[input-engine] CreateWindowExW failed: {}",
                    windows::Win32::Foundation::GetLastError().0);
                CAPTURING.store(false, Ordering::SeqCst);
                return;
            }
        };

        let devices = [
            RAWINPUTDEVICE {
                usUsagePage: 0x01,
                usUsage: 0x06,
                dwFlags: windows::Win32::UI::Input::RAWINPUTDEVICE_FLAGS(0),
                hwndTarget: hwnd,
            },
            RAWINPUTDEVICE {
                usUsagePage: 0x01,
                usUsage: 0x02,
                dwFlags: windows::Win32::UI::Input::RAWINPUTDEVICE_FLAGS(0),
                hwndTarget: hwnd,
            },
        ];

        let result = RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32);
        if result.is_err() {
            println!("[input-engine] RegisterRawInputDevices failed: {}",
                windows::Win32::Foundation::GetLastError().0);
            let _ = DestroyWindow(hwnd);
            CAPTURING.store(false, Ordering::SeqCst);
            return;
        }

        println!("[input-engine] Capture started — listening for WM_INPUT messages");

        let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = DestroyWindow(hwnd);
        println!("[input-engine] Capture stopped");
    }

    CAPTURING.store(false, Ordering::SeqCst);
}

pub fn start_capture() -> Result<(), String> {
    if CAPTURING.load(Ordering::SeqCst) {
        return Err("Capture is already running".to_string());
    }

    CAPTURING.store(true, Ordering::SeqCst);

    thread::spawn(|| {
        unsafe {
            THREAD_ID = windows::Win32::System::Threading::GetCurrentThreadId();
        }
        capture_thread();
    });

    Ok(())
}

pub fn stop_capture() {
    if !CAPTURING.load(Ordering::SeqCst) {
        return;
    }
    unsafe {
        if THREAD_ID != 0 {
            let _ = PostThreadMessageW(THREAD_ID, WM_QUIT, WPARAM(0), LPARAM(0));
        }
    }
}

pub fn is_capturing() -> bool {
    CAPTURING.load(Ordering::SeqCst)
}
