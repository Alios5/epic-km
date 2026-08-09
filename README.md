# Epic KM

<img src="assets/trace%201.svg" alt="Epic KM logo" width="128" />

**Turn your mouse and keyboard into an Xbox 360 controller.**

## Why Epic KM exists

Some games simply don't support a mouse — they expect a gamepad, period. This is
especially common with console emulators: for example, **Ryujinx** (a Nintendo
Switch emulator) lets you play with a controller, but aiming a camera with a
mouse is either impossible or painfully limited.

Epic KM bridges that gap. It creates a **virtual Xbox 360 controller** on your
PC (via the ViGEmBus driver) and feeds it with your mouse and keyboard:

- **Mouse movement → right stick**: smooth, analog camera control with real
  sensitivity tuning, curves and smoothing — the closest thing to native
  mouse aiming a controller-only game can get.
- **Keyboard keys / mouse buttons → any gamepad input**: buttons, bumpers,
  triggers, D-Pad, stick clicks.
- **Keyboard → left stick**: digital WASD-style movement.

Press a global hotkey (F1 by default) and Epic KM captures your inputs — the
cursor is locked and hidden, your keystrokes become gamepad reports. Press it
again and your PC is back to normal. To every game and emulator, it looks like
a genuine Xbox 360 pad is plugged in.

## Features

- **Virtual Xbox 360 controller** powered by ViGEmBus, with automatic driver
  detection and guided installation.
- **Clickable gamepad diagram**: click any part of the controller art to assign
  a key or mouse button to it.
- **Full mouse-look tuning**, applied live while you play:
  - global sensitivity **plus per-axis (X/Y) sensitivity** multipliers
  - linear / exponential response curves
  - deadzone, output **smoothing**, polling rate up to 1000 Hz
- **Layout-independent key identification** (scan codes), so bindings stay
  correct on AZERTY, QWERTZ and other non-QWERTY keyboards.
- **Profiles**: save, load and switch between complete mapping configurations.
- **Bilingual interface** (English / French), dark theme, window size
  persistence.

## Requirements

- Windows 10 / 11
- [ViGEmBus](https://github.com/nefarius/ViGEmBus) driver (virtual gamepad
  framework). Epic KM checks for it at startup and points you to the installer
  if it's missing.

## Usage

1. Launch Epic KM — it verifies that ViGEmBus is installed.
2. Open the mapping editor and assign your keys (or mouse buttons) to the
   gamepad inputs. The right stick is driven by the mouse.
3. Tune the stick settings (sensitivity, curve, smoothing…) — changes apply
   instantly, even while a game is running.
4. Press **F1** (or your custom toggle key) to start capturing, and play.
   Press it again to release the mouse and keyboard.

## Tech stack

- **[Tauri 2](https://tauri.app/)** — lightweight native shell (Rust backend,
  WebView2 frontend)
- **SvelteKit 2 / Svelte 5 + TypeScript + Tailwind CSS 4** — user interface
- **`input-engine`** — custom Rust crate doing the heavy lifting:
  - Win32 **Raw Input** + low-level keyboard hook for capture and interception
  - scan-code-based key identification (layout-independent)
  - cursor confinement (`ClipCursor`) while capturing
  - high-frequency emission thread with a 1 ms timer
  - [`vigem-client`](https://crates.io/crates/vigem-client) to talk to ViGEmBus

## Development

```sh
npm install
npm run tauri dev    # dev mode with hot reload
npm run tauri build  # production build + Windows installer
```

Other useful commands:

```sh
npm run check        # svelte-check (types + template validation)
npm run build        # frontend-only build
```
