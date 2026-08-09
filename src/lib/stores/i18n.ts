import { derived, writable } from "svelte/store";

export type Locale = "fr" | "en";

const fr = {
  // Home
  "app.checking": "Vérification des prérequis…",
  "home.ready": "Prérequis installés — l'outil est prêt à démarrer.",
  "home.configure": "Configurer le mapping",
  "home.language": "Langue",
  // Prerequisites missing
  "prereq.title": "Pilote requis : ViGEmBus",
  "prereq.intro": "Pour fonctionner, Epic KM a besoin du pilote virtuel",
  "prereq.purpose": "Il permet de simuler une manette Xbox 360 pour envoyer les inputs du mapping.",
  "prereq.download": "Télécharger et installer",
  "prereq.recheck": "Revérifier",
  "prereq.notResponding":
    "Le pilote est installé mais ne répond pas encore. Si vous venez de l'installer, redémarrez le PC puis cliquez sur « Revérifier ».",
  "prereq.autoDetect": "Détection automatique en cours…",
  "prereq.manualHint":
    "Après l'installation, la détection se fait automatiquement. Vous pouvez aussi cliquer sur « Revérifier ».",
  // Profile editor
  "editor.defaultName": "Défaut",
  "editor.emptyName": "Veuillez saisir un nom de profil",
  "editor.saveSuccess": "Profil « {name} » sauvegardé",
  "editor.saveError": "Échec de la sauvegarde : {error}",
  "editor.saving": "Sauvegarde…",
  // Capture badge + top bar
  "capture.active": "Capture active",
  "capture.inactive": "Capture inactive",
  "topbar.key": "Touche:",
  "topbar.pressKey": "Touche…",
  "topbar.toggleKeyTitle": "Touche de bascule capture",
  "topbar.recent": "Récents…",
  "topbar.profileName": "Nom du profil",
  // Common actions
  "common.save": "Sauvegarder",
  "common.delete": "Supprimer",
  "common.export": "Exporter le profil",
  "common.openFile": "Ouvrir un fichier",
  "common.close": "Fermer",
  "common.cancel": "Annuler",
  "common.confirm": "Confirmer",
  // Window / close confirmation
  "win.minimize": "Réduire",
  "win.maximize": "Agrandir",
  "close.unsavedTitle": "Modifications non sauvegardées",
  "close.unsavedMsg": "Vous avez des modifications non sauvegardées. Voulez-vous les sauvegarder avant de fermer ?",
  "close.saveAndClose": "Sauvegarder et fermer",
  "close.closeWithoutSaving": "Fermer sans sauvegarder",
  "close.confirmTitle": "Fermer l'application",
  "close.confirmMsg": "Êtes-vous sûr de vouloir fermer Epic KM ?",
  // Bottom bar
  "bottombar.triggerThreshold": "Seuil de déclenchement",
  "bottombar.captureKey": "Touche capture",
  "bottombar.hideCursor": "Masquer le curseur",
  // Side panels
  "panel.leftTriggers": "Gâchettes gauche",
  "panel.rightTriggers": "Gâchettes droite",
  "panel.faceButtons": "Boutons A/B/X/Y",
  "panel.leftStick": "Stick gauche",
  "panel.rightStickMouse": "Stick droit (souris)",
  "panel.stickClick": "Clic stick",
  "panel.dpad": "Croix directionnelle",
  "dir.up": "Haut",
  "dir.down": "Bas",
  "dir.left": "Gauche",
  "dir.right": "Droite",
  // Left stick keyboard mapping
  "lsm.title": "Stick gauche (clavier)",
  "lsm.empty": "Aucune touche assignée.",
  // Key capture input
  "keycap.press": "Appuyez sur une touche…",
  // Assignment popover
  "pop.currentKey": "Touche actuelle :",
  "pop.none": "Aucune",
  "pop.pressOrClick": "Appuyez sur une touche ou cliquez…",
  "pop.change": "Changer la touche",
  "pop.assign": "Assigner une touche",
  "pop.clear": "Effacer",
  // Friendly gamepad button names (popover titles / zone tooltips)
  "btn.A": "Bouton A",
  "btn.B": "Bouton B",
  "btn.X": "Bouton X",
  "btn.Y": "Bouton Y",
  "btn.LB": "Bumper gauche (LB)",
  "btn.RB": "Bumper droit (RB)",
  "btn.LT": "Gâchette gauche (LT)",
  "btn.RT": "Gâchette droite (RT)",
  "btn.Back": "Back",
  "btn.Start": "Start",
  "btn.LeftThumb": "Stick gauche (clic)",
  "btn.RightThumb": "Stick droit (clic)",
  "btn.DPadUp": "Croix haut",
  "btn.DPadDown": "Croix bas",
  "btn.DPadLeft": "Croix gauche",
  "btn.DPadRight": "Croix droite",
  // Stick settings
  "stick.sensitivity": "Sensibilité",
  "stick.perAxis": "Par axe (X/Y)",
  "stick.sensitivityX": "Sensibilité X",
  "stick.sensitivityY": "Sensibilité Y",
  "stick.curve": "Courbe",
  "stick.linear": "Linéaire",
  "stick.exponential": "Exponentielle",
  "stick.deadzone": "Zone morte",
  "stick.smoothing": "Lissage",
  "stick.interval": "Intervalle (Hz)",
  "stick.invertY": "Inverser axe Y",
  "stick.invertX": "Inverser axe X",
  // Gamepad diagram
  "diagram.hint": "Cliquez sur un élément de la manette pour lui assigner une touche ou un clic",
  // Delete profile confirmation
  "delete.title": "Supprimer le profil",
  "delete.message": "Voulez-vous vraiment supprimer le profil « {name} » ?",
  "delete.success": "Profil « {name} » supprimé",
  "delete.error": "Échec de la suppression : {error}",
} as const;

export type MessageKey = keyof typeof fr;

const en: Record<MessageKey, string> = {
  "app.checking": "Checking prerequisites…",
  "home.ready": "Prerequisites installed — the tool is ready to go.",
  "home.configure": "Configure mapping",
  "home.language": "Language",
  "prereq.title": "Required driver: ViGEmBus",
  "prereq.intro": "To work, Epic KM needs the virtual driver",
  "prereq.purpose": "It emulates an Xbox 360 controller so the mapping inputs can be sent.",
  "prereq.download": "Download and install",
  "prereq.recheck": "Check again",
  "prereq.notResponding":
    'The driver is installed but not responding yet. If you just installed it, restart your PC, then click "Check again".',
  "prereq.autoDetect": "Auto-detection in progress…",
  "prereq.manualHint":
    'After installation, detection happens automatically. You can also click "Check again".',
  "editor.defaultName": "Default",
  "editor.emptyName": "Please enter a profile name",
  "editor.saveSuccess": 'Profile "{name}" saved',
  "editor.saveError": "Save failed: {error}",
  "editor.saving": "Saving…",
  "capture.active": "Capture on",
  "capture.inactive": "Capture off",
  "topbar.key": "Key:",
  "topbar.pressKey": "Key…",
  "topbar.toggleKeyTitle": "Capture toggle key",
  "topbar.recent": "Recent…",
  "topbar.profileName": "Profile name",
  "common.save": "Save",
  "common.delete": "Delete",
  "common.export": "Export profile",
  "common.openFile": "Open file",
  "common.close": "Close",
  "common.cancel": "Cancel",
  "common.confirm": "Confirm",
  "win.minimize": "Minimize",
  "win.maximize": "Maximize",
  "close.unsavedTitle": "Unsaved changes",
  "close.unsavedMsg": "You have unsaved changes. Do you want to save them before closing?",
  "close.saveAndClose": "Save and close",
  "close.closeWithoutSaving": "Close without saving",
  "close.confirmTitle": "Close application",
  "close.confirmMsg": "Are you sure you want to close Epic KM?",
  "bottombar.triggerThreshold": "Trigger threshold",
  "bottombar.captureKey": "Capture key",
  "bottombar.hideCursor": "Hide cursor",
  "panel.leftTriggers": "Left triggers",
  "panel.rightTriggers": "Right triggers",
  "panel.faceButtons": "A/B/X/Y buttons",
  "panel.leftStick": "Left stick",
  "panel.rightStickMouse": "Right stick (mouse)",
  "panel.stickClick": "Stick click",
  "panel.dpad": "D-Pad",
  "dir.up": "Up",
  "dir.down": "Down",
  "dir.left": "Left",
  "dir.right": "Right",
  "lsm.title": "Left stick (keyboard)",
  "lsm.empty": "No key assigned.",
  "keycap.press": "Press a key…",
  "pop.currentKey": "Current key:",
  "pop.none": "None",
  "pop.pressOrClick": "Press a key or click…",
  "pop.change": "Change key",
  "pop.assign": "Assign a key",
  "pop.clear": "Clear",
  "btn.A": "A button",
  "btn.B": "B button",
  "btn.X": "X button",
  "btn.Y": "Y button",
  "btn.LB": "Left bumper (LB)",
  "btn.RB": "Right bumper (RB)",
  "btn.LT": "Left trigger (LT)",
  "btn.RT": "Right trigger (RT)",
  "btn.Back": "Back",
  "btn.Start": "Start",
  "btn.LeftThumb": "Left stick (click)",
  "btn.RightThumb": "Right stick (click)",
  "btn.DPadUp": "D-Pad up",
  "btn.DPadDown": "D-Pad down",
  "btn.DPadLeft": "D-Pad left",
  "btn.DPadRight": "D-Pad right",
  "stick.sensitivity": "Sensitivity",
  "stick.perAxis": "Per axis (X/Y)",
  "stick.sensitivityX": "Sensitivity X",
  "stick.sensitivityY": "Sensitivity Y",
  "stick.curve": "Curve",
  "stick.linear": "Linear",
  "stick.exponential": "Exponential",
  "stick.deadzone": "Deadzone",
  "stick.smoothing": "Smoothing",
  "stick.interval": "Rate (Hz)",
  "stick.invertY": "Invert Y axis",
  "stick.invertX": "Invert X axis",
  "diagram.hint": "Click a controller element to assign a key or mouse click",
  "delete.title": "Delete profile",
  "delete.message": "Are you sure you want to delete profile \"{name}\"?",
  "delete.success": "Profile \"{name}\" deleted",
  "delete.error": "Delete failed: {error}",
};

const messages: Record<Locale, Record<MessageKey, string>> = { fr, en };

const STORAGE_KEY = "epic-mouse-locale";

function initialLocale(): Locale {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "fr" || saved === "en") return saved;
  } catch {
    // localStorage unavailable — fall back to French
  }
  return "fr";
}

export const locale = writable<Locale>(initialLocale());

export function setLocale(l: Locale) {
  locale.set(l);
  try {
    localStorage.setItem(STORAGE_KEY, l);
  } catch {
    // Non-persistent is fine
  }
}

/**
 * Translate function as a derived store: use `$t("key")` in templates.
 * Supports {param} interpolation: `$t("editor.saveSuccess", { name })`.
 */
export const t = derived(locale, ($locale) => {
  const dict = messages[$locale];
  return (key: MessageKey, params?: Record<string, string>): string => {
    let s: string = dict[key] ?? key;
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        s = s.replaceAll(`{${k}}`, v);
      }
    }
    return s;
  };
});
