import { writable } from "svelte/store";
import type { Locale } from "$lib/stores/i18n";

/**
 * Localized labels for KeyboardEvent.code values.
 *
 * Bindings are stored as physical `code` values (layout-independent), but the
 * user thinks in terms of what is printed on their key caps. The Keyboard API
 * layout map translates a code to the character the key produces in the
 * current layout (e.g. "KeyW" -> "z" on AZERTY).
 */
export const layoutMap = writable<Map<string, string> | null>(null);

// Load the layout map once (Chromium / WebView2). Silently stays null when
// the API is unavailable; labelForCode then falls back to raw codes.
if (typeof navigator !== "undefined") {
  const kb = (navigator as unknown as { keyboard?: { getLayoutMap?: () => Promise<Map<string, string>> } }).keyboard;
  kb?.getLayoutMap?.()
    .then((map) => layoutMap.set(map))
    .catch(() => {});
}

/** Friendly names for keys that are not covered by the layout map. */
const SPECIAL_LABELS: Record<Locale, Record<string, string>> = {
  fr: {
    Space: "Espace",
    Enter: "Entrée",
    NumpadEnter: "Entrée (pavé)",
    Backspace: "Retour",
    Tab: "Tab",
    Escape: "Échap",
    ArrowUp: "↑",
    ArrowDown: "↓",
    ArrowLeft: "←",
    ArrowRight: "→",
    ShiftLeft: "Maj gauche",
    ShiftRight: "Maj droit",
    ControlLeft: "Ctrl gauche",
    ControlRight: "Ctrl droit",
    AltLeft: "Alt gauche",
    AltRight: "Alt Gr",
    MetaLeft: "Super gauche",
    MetaRight: "Super droit",
    CapsLock: "Verr Maj",
    NumLock: "Verr Num",
    ScrollLock: "Arrêt défil",
    Insert: "Inser",
    Delete: "Suppr",
    Home: "Début",
    End: "Fin",
    PageUp: "Pg préc",
    PageDown: "Pg suiv",
    ContextMenu: "Menu",
    PrintScreen: "Impr. écran",
    Pause: "Pause",
    MouseLeft: "Clic gauche",
    MouseRight: "Clic droit",
    MouseMiddle: "Clic molette",
    MouseX1: "Souris X1",
    MouseX2: "Souris X2",
  },
  en: {
    Space: "Space",
    Enter: "Enter",
    NumpadEnter: "Enter (numpad)",
    Backspace: "Backspace",
    Tab: "Tab",
    Escape: "Esc",
    ArrowUp: "↑",
    ArrowDown: "↓",
    ArrowLeft: "←",
    ArrowRight: "→",
    ShiftLeft: "Left Shift",
    ShiftRight: "Right Shift",
    ControlLeft: "Left Ctrl",
    ControlRight: "Right Ctrl",
    AltLeft: "Left Alt",
    AltRight: "Alt Gr",
    MetaLeft: "Left Super",
    MetaRight: "Right Super",
    CapsLock: "Caps Lock",
    NumLock: "Num Lock",
    ScrollLock: "Scroll Lock",
    Insert: "Insert",
    Delete: "Delete",
    Home: "Home",
    End: "End",
    PageUp: "Page Up",
    PageDown: "Page Down",
    ContextMenu: "Menu",
    PrintScreen: "Print Screen",
    Pause: "Pause",
    MouseLeft: "Left click",
    MouseRight: "Right click",
    MouseMiddle: "Middle click",
    MouseX1: "Mouse X1",
    MouseX2: "Mouse X2",
  },
};

/**
 * Human-readable label for a stored KeyboardEvent.code, adapted to the
 * user's keyboard layout and UI language when possible.
 */
export function labelForCode(
  code: string,
  map: Map<string, string> | null,
  loc: Locale = "fr",
): string {
  if (!code) return "—";
  const special = SPECIAL_LABELS[loc][code];
  if (special) return special;
  const localized = map?.get(code);
  if (localized) {
    return localized.length === 1 ? localized.toUpperCase() : localized;
  }
  return code;
}
