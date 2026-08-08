import { invoke } from "@tauri-apps/api/core";
import { get } from "svelte/store";
import { profile, activeProfileName, markClean, type Profile, type StickConfig } from "./profile";

export async function saveProfile(name: string): Promise<void> {
  const data = get(profile);
  await invoke("save_profile", { name, data });
  activeProfileName.set(name);
  markClean();
}

// Older saved profiles may lack recently added fields — fill in defaults
function normalizeStick(s: StickConfig, smoothingDefault: number): StickConfig {
  return {
    ...s,
    sensitivityX: s.sensitivityX ?? 1.0,
    sensitivityY: s.sensitivityY ?? 1.0,
    smoothing: s.smoothing ?? smoothingDefault,
  };
}

export async function loadProfile(name: string): Promise<Profile> {
  // Suspend the capture hotkey while swapping profiles, then push the loaded
  // profile to the engine — this re-registers the hotkey with the new key.
  await invoke("suspend_hotkey").catch(() => {});
  const data = await invoke<Profile>("load_profile", { name });
  const normalized: Profile = {
    ...data,
    leftStick: normalizeStick(data.leftStick, 0.0),
    rightStick: normalizeStick(data.rightStick, 0.3),
  };
  profile.set(normalized);
  activeProfileName.set(name);
  await invoke("reload_profile", { profile: normalized }).catch(() => {});
  return normalized;
}

export async function listProfiles(): Promise<string[]> {
  return await invoke<string[]>("list_profiles");
}

export async function deleteProfile(name: string): Promise<void> {
  await invoke("delete_profile", { name });
}
