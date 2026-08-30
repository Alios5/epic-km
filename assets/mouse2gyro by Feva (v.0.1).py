import socket
import struct
import time
import threading
import tkinter as tk
from tkinter import ttk
import ctypes
import random

MAGIC_S = b"DSUS"
MAGIC_C = b"DSUC"
PROTO   = 1001

TYPE_VERSION = 0x100000
TYPE_PORTS   = 0x100001
TYPE_DATA    = 0x100002

SERVER_ID = random.randint(1, 0xFFFFFFFF)

_CRC_TABLE = []
for _i in range(256):
    _c = _i
    for _ in range(8):
        _c = (_c >> 1) ^ (0xEDB88320 if _c & 1 else 0)
    _CRC_TABLE.append(_c)

def _crc32(data: bytes) -> int:
    crc = 0xFFFFFFFF
    for b in data:
        crc = _CRC_TABLE[(crc ^ b) & 0xFF] ^ (crc >> 8)
    return crc ^ 0xFFFFFFFF

def _make(msg_type: int, payload: bytes) -> bytes:
    body   = struct.pack("<I", msg_type) + payload
    length = len(body)
    hdr = struct.pack("<4sHHII", MAGIC_S, PROTO, length, 0, SERVER_ID)
    crc = _crc32(hdr + body)
    hdr = struct.pack("<4sHHII", MAGIC_S, PROTO, length, crc, SERVER_ID)
    return hdr + body

def pkt_version() -> bytes:
    return _make(TYPE_VERSION, struct.pack("<H", PROTO))

def _slot_info(slot: int) -> bytes:
    return struct.pack("<BBBB", slot, 2, 2, 1) + \
           b"\x00\x00\x00\x00\x00\x01" + \
           struct.pack("<B", 0xEF)

def pkt_ports(slot: int) -> bytes:
    return _make(TYPE_PORTS, _slot_info(slot) + b"\x00")

def pkt_data(pitch: float, yaw: float, roll: float,
             ax: float, ay: float, az: float,
             pkt_num: int) -> bytes:
    ts = int(time.time() * 1_000_000) & 0xFFFFFFFFFFFFFFFF
    p  = _slot_info(0)
    p += struct.pack("<B", 1)
    p += struct.pack("<I", pkt_num)
    p += struct.pack("<BBBB", 0, 0, 0, 0)
    p += struct.pack("<BBBB", 0x80, 0x80, 0x80, 0x80)
    p += bytes(12)
    p += struct.pack("<BBHh", 0, 0, 0, 0)
    p += struct.pack("<BBHh", 0, 0, 0, 0)
    p += struct.pack("<Q", ts)
    p += struct.pack("<fff", ax, ay, az)
    p += struct.pack("<fff", pitch, yaw, roll)
    return _make(TYPE_DATA, p)


WM_INPUT                = 0x00FF
RID_INPUT               = 0x10000003
RIDEV_INPUTSINK         = 0x00000100
HID_USAGE_PAGE_GENERIC  = 0x01
HID_USAGE_GENERIC_MOUSE = 0x02

class _RAWINPUTDEVICE(ctypes.Structure):
    _fields_ = [("usUsagePage", ctypes.c_ushort), ("usUsage", ctypes.c_ushort),
                ("dwFlags", ctypes.c_ulong),       ("hwndTarget", ctypes.c_void_p)]

class _RAWINPUTHEADER(ctypes.Structure):
    _fields_ = [("dwType",  ctypes.c_ulong),
                ("dwSize",  ctypes.c_ulong),
                ("hDevice", ctypes.c_uint64),
                ("wParam",  ctypes.c_uint64)]

class _RAWMOUSE(ctypes.Structure):
    _fields_ = [("usFlags",            ctypes.c_ushort),
                ("usButtonFlags",      ctypes.c_ushort),
                ("usButtonData",       ctypes.c_ushort),
                ("ulRawButtons",       ctypes.c_ulong),
                ("lLastX",             ctypes.c_long),
                ("lLastY",             ctypes.c_long),
                ("ulExtraInformation", ctypes.c_ulong)]

class _RAWINPUT(ctypes.Structure):
    _fields_ = [("header", _RAWINPUTHEADER),
                ("mouse",  _RAWMOUSE)]


class MouseCapture:
    def __init__(self, callback):
        self.callback = callback

    def start(self):
        threading.Thread(target=self._run, daemon=True).start()

    def _run(self):
        import ctypes.wintypes as wt
        hInst = ctypes.windll.kernel32.GetModuleHandleW(None)
        PROC  = ctypes.WINFUNCTYPE(ctypes.c_long, wt.HWND, wt.UINT, wt.WPARAM, wt.LPARAM)

        def wnd_proc(hwnd, msg, wp, lp):
            if msg == WM_INPUT:
                sz = ctypes.c_uint(0)
                ctypes.windll.user32.GetRawInputData(
                    lp, RID_INPUT, None, ctypes.byref(sz), ctypes.sizeof(_RAWINPUTHEADER))
                buf = ctypes.create_string_buffer(sz.value)
                ctypes.windll.user32.GetRawInputData(
                    lp, RID_INPUT, buf, ctypes.byref(sz), ctypes.sizeof(_RAWINPUTHEADER))
                ri = _RAWINPUT.from_buffer_copy(buf)
                if ri.header.dwType == 0:
                    dx, dy = ri.mouse.lLastX, ri.mouse.lLastY
                    if dx or dy:
                        self.callback(dx, dy)
            return ctypes.windll.user32.DefWindowProcW(hwnd, msg, wp, lp)

        proc_c = PROC(wnd_proc)

        class WC(ctypes.Structure):
            _fields_ = [("style", wt.UINT), ("lpfnWndProc", PROC),
                        ("cbClsExtra", ctypes.c_int), ("cbWndExtra", ctypes.c_int),
                        ("hInstance", wt.HINSTANCE), ("hIcon", wt.HICON),
                        ("hCursor", wt.HANDLE), ("hbrBackground", wt.HBRUSH),
                        ("lpszMenuName", wt.LPCWSTR), ("lpszClassName", wt.LPCWSTR)]

        wc = WC(); wc.lpfnWndProc = proc_c; wc.hInstance = hInst
        wc.lpszClassName = "M2GCapture"
        ctypes.windll.user32.RegisterClassW(ctypes.byref(wc))
        hwnd = ctypes.windll.user32.CreateWindowExW(
            0x08000000, "M2GCapture", "", 0, 0, 0, 0, 0, None, None, hInst, None)

        rid = _RAWINPUTDEVICE()
        rid.usUsagePage = HID_USAGE_PAGE_GENERIC
        rid.usUsage     = HID_USAGE_GENERIC_MOUSE
        rid.dwFlags     = RIDEV_INPUTSINK
        rid.hwndTarget  = hwnd
        ctypes.windll.user32.RegisterRawInputDevices(
            ctypes.byref(rid), 1, ctypes.sizeof(rid))

        msg = ctypes.wintypes.MSG()
        while True:
            if ctypes.windll.user32.PeekMessageW(ctypes.byref(msg), None, 0, 0, 1):
                ctypes.windll.user32.TranslateMessage(ctypes.byref(msg))
                ctypes.windll.user32.DispatchMessageW(ctypes.byref(msg))
            else:
                time.sleep(0.001)


class DSUServer:
    def __init__(self):
        self.enabled        = False
        self.sens_h         = 6.0
        self.sens_v         = 4.0
        self.smoothing      = 0.0
        self.port           = 26760
        self.ay_lock        = -1.27   # valeur ay anti-recalibration
        self.recalib_delay  = 3.0     # secondes d'immobilité avant recalibration naturelle (0 = jamais)

        self._pitch     = 0.0
        self._yaw       = 0.0
        self._roll      = 0.0
        self._lock      = threading.Lock()
        self._pnum      = 0
        self._last_move = 0.0

        self._subs      = {}
        self._last_addr = None
        self._sub_lock  = threading.Lock()

        self._sock    = None
        self._alive   = False

        MouseCapture(self._on_delta).start()

    def start(self):
        self._alive = True
        self._sock  = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self._sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self._sock.bind(("0.0.0.0", self.port))
        self._sock.settimeout(0.5)
        threading.Thread(target=self._recv, daemon=True).start()
        threading.Thread(target=self._send, daemon=True).start()

    def restart(self, port: int):
        self._alive = False
        try: self._sock.close()
        except: pass
        time.sleep(0.4)
        self.port = port
        self.start()

    def _on_delta(self, dx: int, dy: int):
        if not self.enabled:
            return
        with self._lock:
            s = self.smoothing
            self._yaw   = self._yaw   * s + dx   * self.sens_h * (1 - s)
            self._pitch = self._pitch * s + (-dy) * self.sens_v * (1 - s)
            self._roll  = 0.0
            self._last_move = time.perf_counter()

    def _recv(self):
        while self._alive:
            try:
                data, addr = self._sock.recvfrom(1024)
            except socket.timeout:
                continue
            except:
                break
            if len(data) < 20 or data[:4] != MAGIC_C:
                continue
            msg_type = struct.unpack_from("<I", data, 16)[0]
            if msg_type == TYPE_VERSION:
                try: self._sock.sendto(pkt_version(), addr)
                except: pass
            elif msg_type == TYPE_PORTS:
                try: self._sock.sendto(pkt_ports(0), addr)
                except: pass
            elif msg_type == TYPE_DATA:
                with self._sub_lock:
                    self._subs[addr] = time.time() + 60.0
                    self._last_addr  = addr

    def _send(self):
        interval = 1 / 120
        while self._alive:
            t0 = time.perf_counter()

            with self._lock:
                p, y, r = self._pitch, self._yaw, self._roll
                self._pitch = 0.0
                self._yaw   *= 0.75
                self._roll  *= 0.75
                idle_time = t0 - self._last_move

            noise_a = 0.02
            ax = random.uniform(-noise_a, noise_a)
            az = random.uniform(-noise_a, noise_a)

            # Si recalib_delay > 0 et souris immobile depuis recalib_delay secondes
            # → ay = -1.0 pour laisser Eden recalibrer naturellement (recentrage cam)
            if self.recalib_delay > 0 and idle_time >= self.recalib_delay:
                ay = -1.0 + random.uniform(-noise_a, noise_a)
            else:
                ay = self.ay_lock + random.uniform(-noise_a, noise_a)

            now = time.time()
            pkt = pkt_data(p, y, r, ax, ay, az, self._pnum)
            self._pnum += 1

            with self._sub_lock:
                dead = [a for a, exp in self._subs.items() if exp < now]
                for a in dead: del self._subs[a]
                subs = list(self._subs.keys())
                if not subs and self._last_addr:
                    subs = [self._last_addr]

            for addr in subs:
                try: self._sock.sendto(pkt, addr)
                except: pass

            rem = interval - (time.perf_counter() - t0)
            if rem > 0: time.sleep(rem)

    def toggle(self) -> bool:
        self.enabled = not self.enabled
        return self.enabled

    def reset_cam(self):
        with self._lock:
            self._pitch = 0.0
            self._yaw   = 0.0
            self._roll  = 0.0

    def shake(self):
        def _do():
            end = time.time() + 5.0
            while time.time() < end:
                with self._lock:
                    self._yaw = 500.0
                time.sleep(0.1)
                with self._lock:
                    self._yaw = -500.0
                time.sleep(0.1)
            with self._lock:
                self._pitch = 0.0
                self._yaw   = 0.0
                self._roll  = 0.0
        threading.Thread(target=_do, daemon=True).start()


BG   = "#1e1e2e"
FG   = "#cdd6f4"
DIM  = "#6c7086"
BLUE = "#89b4fa"
GRN  = "#a6e3a1"
RED  = "#f38ba8"
YLW  = "#f9e2af"
CARD = "#313244"
HOV  = "#45475a"

class App(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("Mouse2Gyro by Feva")
        self.resizable(False, False)
        self.configure(bg=BG)
        self.srv = DSUServer()
        self.srv.start()
        threading.Thread(target=self._hotkey, daemon=True).start()
        self._ui()

    def _hotkey(self):
        prev_f8 = False
        prev_5  = False
        while True:
            pressed_f8 = bool(ctypes.windll.user32.GetAsyncKeyState(0xBB) & 0x8000)
            pressed_5  = bool(ctypes.windll.user32.GetAsyncKeyState(0x35) & 0x8000)
            if pressed_f8 and not prev_f8:
                self.after(0, self._set_state, self.srv.toggle())
            if pressed_5 and not prev_5:
                self.after(0, self._shake)
            prev_f8 = pressed_f8
            prev_5  = pressed_5
            time.sleep(0.05)

    def _set_state(self, on: bool):
        if on:
            self.lbl_status.config(text="● ACTIVE", fg=GRN)
            self.btn_toggle.config(text="Disable  (=)")
        else:
            self.lbl_status.config(text="○ INACTIVE", fg=RED)
            self.btn_toggle.config(text="Enable  (=)")

    def _ui(self):
        P = dict(padx=16, pady=5)
        tk.Label(self, text="Mouse2Gyro by Feva", font=("Segoe UI", 16, "bold"),
                 fg=FG, bg=BG).pack(pady=(18, 2))
        tk.Label(self, text="Mouse Delta → Gyroscope",
                 font=("Segoe UI", 9), fg=DIM, bg=BG).pack()
        ttk.Separator(self, orient="horizontal").pack(fill="x", pady=10)

        self.lbl_status = tk.Label(self, text="○ INACTIVE",
            font=("Segoe UI", 13, "bold"), fg=RED, bg=BG)
        self.lbl_status.pack(**P)

        self.btn_toggle = tk.Button(self, text="Enable  (=)",
            font=("Segoe UI", 10, "bold"), bg=CARD, fg=FG,
            activebackground=HOV, relief="flat", bd=0,
            padx=14, pady=7,
            command=lambda: self._set_state(self.srv.toggle()))
        self.btn_toggle.pack(**P)

        tk.Button(self, text="Shake! (press 5)",
            font=("Segoe UI", 9), bg="#45475a", fg=YLW,
            activebackground=HOV, relief="flat", bd=0,
            padx=10, pady=5,
            command=self._shake).pack(**P)

        tk.Label(self,
            text="Press 5 to sync motion with the emulator",
            font=("Segoe UI", 8), fg=DIM, bg=BG).pack()

        ttk.Separator(self, orient="horizontal").pack(fill="x", pady=8)

        self._slider("Horizontal sensitivity  (yaw)",
                     1, 10, self.srv.sens_h, self._chg_sens_h, fmt="{:.0f}")
        self._slider("Vertical sensitivity  (pitch)",
                     1, 10, self.srv.sens_v, self._chg_sens_v, fmt="{:.0f}")

        ttk.Separator(self, orient="horizontal").pack(fill="x", pady=8)

        self._slider("Anti-recalibration  (ay)",
                     -2.0, -1.0, self.srv.ay_lock,
                     lambda v: setattr(self.srv, 'ay_lock', v),
                     res=0.01, fmt="{:.2f}")

        self._slider("Natural recalib. after  (sec, 0=never)",
                     0, 10, self.srv.recalib_delay,
                     lambda v: setattr(self.srv, 'recalib_delay', v),
                     res=0.5, fmt="{:.1f}")

        ttk.Separator(self, orient="horizontal").pack(fill="x", pady=8)

        tk.Label(self, text="Eden Setup",
                 font=("Segoe UI", 10, "bold"), fg=YLW, bg=BG
                 ).pack(anchor="w", padx=16)
        tk.Label(self,
            text=(
                "- Launch Eden\n"
                "- Go to Settings > Controls\n"
                "- Click on Configure (under the controller/motion options)\n"
                "- In Server, enter: 127.0.0.1\n"
                "- In Port, enter: 26760\n"
                "- Click on Add Server\n"
                "- Launch Mouse2Gyro\n"
                "- Press = to enable it\n"
                "- Return to Eden\n"
                "- Click on Test\n"
                "- Press OK\n"
                "- Close Mouse2Gyro\n"
                "- Then reopen it\n"
                "- Press 5 or (\n"
                "- Use Alt + Tab to return to the Controls window\n"
                "- Click directly on Motion 1\n"
                "- If everything is working, \"CemuhookUDP\" will appear under Motion 1\n"
                "\n"
                "You can now play Splatoon using your mouse as a gyroscope!\n"
                "For every reuse, always launch Mouse2Gyro after the emulator."
            ),
            font=("Consolas", 8), fg="#a6adc8", bg=BG, justify="left"
        ).pack(anchor="w", padx=20, pady=(4, 10))

        pf = tk.Frame(self, bg=BG); pf.pack(anchor="w", padx=16, pady=(0, 16))
        tk.Label(pf, text="UDP Port:", font=("Segoe UI", 9),
                 fg="#bac2de", bg=BG).pack(side="left")
        self._port_v = tk.StringVar(value="26760")
        tk.Entry(pf, textvariable=self._port_v, width=7,
                 bg=CARD, fg=FG, insertbackground="white",
                 relief="flat", font=("Consolas", 9)).pack(side="left", padx=6)
        tk.Button(pf, text="Apply", bg=HOV, fg=FG,
                  relief="flat", padx=8, pady=2,
                  command=self._apply_port).pack(side="left")

    def _slider(self, label, lo, hi, init, cmd, res=1, fmt="{:.0f}"):
        tk.Label(self, text=label, font=("Segoe UI", 9),
                 fg="#bac2de", bg=BG).pack()
        f = tk.Frame(self, bg=BG); f.pack(padx=16, pady=4)
        var = tk.DoubleVar(value=init)
        lbl = tk.Label(f, text=fmt.format(init), width=6,
                       font=("Segoe UI", 9, "bold"), fg=BLUE, bg=BG)
        lbl.pack(side="right")
        def on_change(v):
            lbl.config(text=fmt.format(float(v)))
            cmd(float(v))
        tk.Scale(f, from_=lo, to=hi, resolution=res,
                 orient="horizontal", length=230, variable=var,
                 bg=BG, fg=FG, troughcolor=CARD,
                 highlightthickness=0, showvalue=False,
                 command=on_change).pack(side="left")

    def _chg_sens_h(self, v): self.srv.sens_h = v
    def _chg_sens_v(self, v): self.srv.sens_v = v
    def _shake(self):         self.srv.shake()

    def _apply_port(self):
        try: self.srv.restart(int(self._port_v.get()))
        except ValueError: pass


if __name__ == "__main__":
    app = App()
    app.mainloop()
