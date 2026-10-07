import { useCallback, useEffect, useSyncExternalStore } from "react";
import { onEvent, settingsApi } from "@/lib/tauri";
import type { Settings, SettingsPatch } from "@/types/settings";

/*
 * One shared copy of the settings for the whole window. It loads once, follows
 * `settings://changed`, and applies updates optimistically.
 */
let current: Settings | null = null;
let loading: Promise<void> | null = null;
let stopListening: (() => void) | null = null;
const subscribers = new Set<() => void>();

function publish(next: Settings) {
  current = next;
  subscribers.forEach((notify) => notify());
}

function ensureLoaded() {
  if (!stopListening) stopListening = onEvent("settings://changed", publish);
  loading ??= settingsApi
    .get()
    .then(publish)
    .catch((error) => {
      console.error("Could not load settings", error);
      loading = null;
    });
}

function subscribe(notify: () => void) {
  subscribers.add(notify);
  return () => subscribers.delete(notify);
}

export interface UseSettings {
  /** `null` until the first load finishes. */
  settings: Settings | null;
  loaded: boolean;
  update: (patch: SettingsPatch) => Promise<void>;
  reset: () => Promise<void>;
}

export function useSettings(): UseSettings {
  const settings = useSyncExternalStore(subscribe, () => current);

  useEffect(ensureLoaded, []);

  const update = useCallback(async (patch: SettingsPatch) => {
    const previous = current;
    if (previous) publish({ ...previous, ...patch });
    try {
      publish(await settingsApi.set(patch));
    } catch (error) {
      console.error("Could not save settings", error);
      if (previous) publish(previous);
    }
  }, []);

  const reset = useCallback(async () => {
    publish(await settingsApi.reset());
  }, []);

  return { settings, loaded: settings !== null, update, reset };
}
