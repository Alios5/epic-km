import { writable } from "svelte/store";

export type StickCurve = "linear" | "exponential";

export const hasUnsavedChanges = writable<boolean>(false);

export function markDirty() {
  hasUnsavedChanges.set(true);
}

export function markClean() {
  hasUnsavedChanges.set(false);
}

export interface KeyboardMapping {
  id: string;
  key: string;
  button: string;
}

export type StickDirection = "up" | "down" | "left" | "right";

export type AxisInputMode = "analog" | "gyroscope";

export type ControllerType = "xbox360" | "ds4";

/** Which axis the DS4 accelerometer reports as 1 g at rest (see profile.rs). */
export type GyroRestAccel = "neg_y" | "pos_y" | "neg_z" | "pos_z" | "zero";

export interface KeyboardStickMapping {
  id: string;
  key: string;
  direction: StickDirection;
}

export interface StickConfig {
  sensitivity: number;
  /** Extra per-axis multipliers on top of the global sensitivity */
  sensitivityX: number;
  sensitivityY: number;
  curve: StickCurve;
  deadzone: number;
  /** Output smoothing amount: 0 = off, up to 0.95 */
  smoothing: number;
  invertY: boolean;
  invertX: boolean;
  refreshInterval: number;
}

export interface Profile {
  keyboardToButton: KeyboardMapping[];
  keyboardToLeftStick: KeyboardStickMapping[];
  /** Independent per-axis mode for the mouse-driven right stick:
   * "analog" = velocity-based, snaps to 0 when mouse stops;
   * "gyroscope" = accumulated position, holds when mouse stops. */
  rightStickXMode: AxisInputMode;
  rightStickYMode: AxisInputMode;
  leftStick: StickConfig;
  rightStick: StickConfig;
  triggerThreshold: number;
  captureToggleKey: string;
  /** Hide the OS cursor while capture mode is active */
  hideCursor: boolean;
  /** Which virtual controller to emulate (Xbox 360 or DualShock 4) */
  controllerType: ControllerType;
  /** DS4 gyro rest-offset compensation in raw LSB (16 LSB = 1 °/s):
   * pre-added so readers that subtract their own assumed bias see exactly
   * 0 °/s at rest. Defaults match the ViGEmBus calibration blob (pitch 1). */
  gyroBiasPitch: number;
  gyroBiasYaw: number;
  /** Rest gravity axis for the DS4 accelerometer (horizon-correction input). */
  gyroRestAccel: GyroRestAccel;
  /** Serve motion over Cemuhook/DSU (UDP 26760): emulators read the gyro as
   * plain floats, bypassing HID calibration entirely (no rest drift). */
  dsuEnabled: boolean;
}

export const GAMEPAD_BUTTONS = [
  "A",
  "B",
  "X",
  "Y",
  "DPadUp",
  "DPadDown",
  "DPadLeft",
  "DPadRight",
  "LB",
  "RB",
  "LT",
  "RT",
  "Start",
  "Back",
  "LeftThumb",
  "RightThumb",
] as const;

// Buttons that can be mapped to a keyboard key
export const MAPPABLE_BUTTONS = [
  "A",
  "B",
  "X",
  "Y",
  "DPadUp",
  "DPadDown",
  "DPadLeft",
  "DPadRight",
  "LB",
  "RB",
  "LT",
  "RT",
  "Start",
  "Back",
  "LeftThumb",
  "RightThumb",
] as const;

export const PROFILES = ["Défaut", "FPS", "Course"] as readonly string[];

export const activeProfileName = writable<string>("Défaut");

function defaultProfile(): Profile {
  return {
    keyboardToButton: [
      { id: "1", key: "Space", button: "A" },
      { id: "2", key: "KeyE", button: "B" },
      { id: "3", key: "KeyQ", button: "X" },
      { id: "4", key: "ShiftLeft", button: "RB" },
    ],
    keyboardToLeftStick: [
      { id: "ls1", key: "KeyW", direction: "up" },
      { id: "ls2", key: "KeyS", direction: "down" },
      { id: "ls3", key: "KeyA", direction: "left" },
      { id: "ls4", key: "KeyD", direction: "right" },
    ],
    rightStickXMode: "analog",
    rightStickYMode: "analog",
    leftStick: {
      sensitivity: 1.0,
      sensitivityX: 1.0,
      sensitivityY: 1.0,
      curve: "linear",
      deadzone: 0.1,
      smoothing: 0.0,
      invertY: false,
      invertX: false,
      refreshInterval: 60,
    },
    rightStick: {
      sensitivity: 1.5,
      sensitivityX: 1.0,
      sensitivityY: 1.0,
      curve: "linear",
      deadzone: 0.02,
      smoothing: 0.3,
      invertY: false,
      invertX: false,
      refreshInterval: 240,
    },
    triggerThreshold: 0.5,
    captureToggleKey: "F1",
    hideCursor: true,
    controllerType: "xbox360",
    gyroBiasPitch: 1,
    gyroBiasYaw: 0,
    gyroRestAccel: "neg_y",
    dsuEnabled: false,
  };
}

export function getDefaultProfile(): Profile {
  return defaultProfile();
}

export const profile = writable<Profile>(defaultProfile());

// Helper: find the key assigned to a button (or empty)
export function getKeyForButton(
  mappings: KeyboardMapping[],
  button: string,
): string {
  const found = mappings.find((m) => m.button === button);
  return found ? found.key : "";
}

// Helper: assign or update a key for a button
export function setKeyForButton(button: string, key: string) {
  markDirty();
  profile.update((p) => {
    const existing = p.keyboardToButton.find((m) => m.button === button);
    if (existing) {
      return {
        ...p,
        keyboardToButton: p.keyboardToButton.map((m) =>
          m.button === button ? { ...m, key } : m,
        ),
      };
    }
    return {
      ...p,
      keyboardToButton: [
        ...p.keyboardToButton,
        { id: crypto.randomUUID(), key, button },
      ],
    };
  });
}

// Helper: remove a mapping by button name
export function removeKeyForButton(button: string) {
  markDirty();
  profile.update((p) => ({
    ...p,
    keyboardToButton: p.keyboardToButton.filter((m) => m.button !== button),
  }));
}

// Helper: remove a mapping by id
export function removeMappingById(id: string) {
  markDirty();
  profile.update((p) => ({
    ...p,
    keyboardToButton: p.keyboardToButton.filter((m) => m.id !== id),
  }));
}
