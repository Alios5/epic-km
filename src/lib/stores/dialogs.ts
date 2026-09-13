import { writable } from "svelte/store";

// Promise-based replacements for the OS-native Tauri dialogs — every modal
// is rendered inside the app window so it inherits the theme.

export interface ConfirmOptions {
  title: string;
  message: string;
  okLabel?: string;
  cancelLabel?: string;
  kind?: "default" | "warning";
}

interface ConfirmState extends ConfirmOptions {
  open: boolean;
  resolve?: (v: boolean) => void;
}

export const confirmState = writable<ConfirmState>({
  open: false,
  title: "",
  message: "",
});

export function confirmDialog(opts: ConfirmOptions): Promise<boolean> {
  return new Promise((resolve) => {
    confirmState.set({ ...opts, open: true, resolve });
  });
}

export function answerConfirm(v: boolean) {
  confirmState.update((s) => {
    const r = s.resolve;
    if (r) queueMicrotask(() => r(v));
    return { ...s, open: false, resolve: undefined };
  });
}

// ---------------------------------------------------------------------------

export interface FileDialogOptions {
  mode: "open" | "save";
  title: string;
  /** Restrict listing/selection to this extension (e.g. "json"); in save
   *  mode it is appended to the filename when missing. */
  extension?: string;
  defaultFilename?: string;
  okLabel?: string;
  cancelLabel?: string;
}

interface FileDialogState extends FileDialogOptions {
  open: boolean;
  resolve?: (v: string | null) => void;
}

export const fileDialogState = writable<FileDialogState>({
  open: false,
  mode: "open",
  title: "",
});

export function fileDialog(opts: FileDialogOptions): Promise<string | null> {
  return new Promise((resolve) => {
    fileDialogState.set({ ...opts, open: true, resolve });
  });
}

export function answerFileDialog(v: string | null) {
  fileDialogState.update((s) => {
    const r = s.resolve;
    if (r) queueMicrotask(() => r(v));
    return { ...s, open: false, resolve: undefined };
  });
}
