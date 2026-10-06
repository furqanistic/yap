/**
 * Typed wrappers around Tauri `invoke` and events.
 *
 * Outside Tauri (plain `npm run dev` in a browser) commands fall back to
 * `localStorage` where that makes sense, so the UI still works.
 */
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getPlatform } from "@/lib/platform";
import type { DiskSpace, ModelEntry, ModelId, ModelProgressEvent, ModelStateEvent } from "@/types/models";
import type { Settings, SettingsPatch } from "@/types/settings";

export { isTauri };

/** Event names emitted by the Rust backend, with their payloads. */
export interface EventMap {
  "settings://changed": Settings;
  "models://progress": ModelProgressEvent;
  "models://state": ModelStateEvent;
}

type EventName = keyof EventMap;

// Browser fallback: events emitted locally so every subscriber stays in sync.
const localEvents = new EventTarget();

function emitLocal<E extends EventName>(name: E, payload: EventMap[E]) {
  localEvents.dispatchEvent(new CustomEvent(name, { detail: payload }));
}

/**
 * Subscribes to a backend event. Returns a function that unsubscribes, which
 * is safe to call before the underlying Tauri listener has finished attaching.
 */
export function onEvent<E extends EventName>(name: E, handler: (payload: EventMap[E]) => void): () => void {
  if (!isTauri()) {
    const listener = (event: Event) => handler((event as CustomEvent<EventMap[E]>).detail);
    localEvents.addEventListener(name, listener);
    return () => localEvents.removeEventListener(name, listener);
  }

  let disposed = false;
  let stop: (() => void) | undefined;
  listen<EventMap[E]>(name, (event) => handler(event.payload)).then((unlisten) => {
    if (disposed) unlisten();
    else stop = unlisten;
  });
  return () => {
    disposed = true;
    stop?.();
  };
}

// ---------- Settings ----------

const STORAGE_KEY = "yap.settings";

/** Browser-only defaults. Inside Tauri, Rust owns the defaults. */
const BROWSER_DEFAULTS: Settings = {
  version: 1,
  holdShortcut: getPlatform() === "macos" ? ["⌥", "Space"] : ["Ctrl", "Space"],
  microphone: "default",
  showRecordingIndicator: true,
  autoPaste: true,
  languageMode: "auto",
  model: "whisper-small",
  computeDevice: "auto",
  launchAtLogin: true,
  startMinimized: false,
};

function readBrowserSettings(): Settings {
  try {
    const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}");
    return { ...BROWSER_DEFAULTS, ...saved, version: 1 };
  } catch {
    return BROWSER_DEFAULTS;
  }
}

function writeBrowserSettings(settings: Settings): Settings {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch {
    // Storage can be unavailable (private mode); keep the in-memory value.
  }
  emitLocal("settings://changed", settings);
  return settings;
}

export const settingsApi = {
  get(): Promise<Settings> {
    if (!isTauri()) return Promise.resolve(readBrowserSettings());
    return invoke<Settings>("settings_get");
  },

  set(patch: SettingsPatch): Promise<Settings> {
    if (!isTauri()) return Promise.resolve(writeBrowserSettings({ ...readBrowserSettings(), ...patch }));
    return invoke<Settings>("settings_set", { patch });
  },

  reset(): Promise<Settings> {
    if (!isTauri()) return Promise.resolve(writeBrowserSettings(BROWSER_DEFAULTS));
    return invoke<Settings>("settings_reset");
  },
};

// ---------- Models ----------

function desktopOnly(): Promise<never> {
  return Promise.reject({ kind: "io", message: "Models can only be downloaded in the desktop app." });
}

/** Rejects with a `ModelError` (`{ kind, message }`) on failure. */
export const modelsApi = {
  /** Every catalog model with its download state. Empty outside the desktop app. */
  list(): Promise<ModelEntry[]> {
    if (!isTauri()) return Promise.resolve([]);
    return invoke<ModelEntry[]>("models_list");
  },

  dir(): Promise<string> {
    if (!isTauri()) return Promise.resolve("");
    return invoke<string>("models_dir");
  },

  /** Free space compared with what the download still needs. */
  diskSpace(id: ModelId): Promise<DiskSpace> {
    if (!isTauri()) return desktopOnly();
    return invoke<DiskSpace>("models_disk_space", { id });
  },

  /** Starts a download and resolves right away; progress arrives as events. */
  download(id: ModelId): Promise<void> {
    if (!isTauri()) return desktopOnly();
    return invoke("models_download", { id });
  },

  /** Stops a download, keeping the partial file so it can resume. */
  cancel(id: ModelId): Promise<void> {
    if (!isTauri()) return Promise.resolve();
    return invoke("models_cancel", { id });
  },

  remove(id: ModelId): Promise<void> {
    if (!isTauri()) return desktopOnly();
    return invoke("models_delete", { id });
  },

  openFolder(): Promise<void> {
    if (!isTauri()) return desktopOnly();
    return invoke("models_open_folder");
  },
};
