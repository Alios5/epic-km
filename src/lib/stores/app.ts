import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export type CheckState = "idle" | "checking" | "done";
export type VigemStatus = "unknown" | "ok" | "not-installed" | "not-responding";

export const checkState = writable<CheckState>("idle");
export const prerequisitesInstalled = writable<boolean>(false);
export const vigemStatus = writable<VigemStatus>("unknown");
export const engineRunning = writable<boolean>(false);
export const captureModeActive = writable<boolean>(false);

// ViGEmBus driver repository
export const VIGEMBUS_REPO_URL =
  "https://github.com/ViGEm/ViGEmBus/releases/latest";

// Calls the Rust backend to check if ViGEmBus driver service is installed
export async function checkPrerequisites(): Promise<void> {
  checkState.set("checking");
  try {
    const status = await invoke<VigemStatus>("check_vigembus");
    vigemStatus.set(status);
    prerequisitesInstalled.set(status === "ok");
  } catch (e) {
    console.error("Failed to check ViGEmBus:", e);
    vigemStatus.set("not-installed");
    prerequisitesInstalled.set(false);
  }
  checkState.set("done");
}
