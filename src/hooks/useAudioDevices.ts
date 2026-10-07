import { useCallback, useEffect, useState } from "react";
import { audioApi } from "@/lib/tauri";
import type { AudioError, InputDevice } from "@/types/audio";

export interface UseAudioDevices {
  devices: InputDevice[];
  /** Set when the device list couldn't be read. */
  error: AudioError | null;
  refresh: () => void;
}

/**
 * Microphones reported by the OS (through Rust, so names show up without a
 * permission prompt). Re-lists when the window regains focus, which catches
 * devices plugged in or removed while Yap was in the background.
 */
export function useAudioDevices(): UseAudioDevices {
  const [devices, setDevices] = useState<InputDevice[]>([]);
  const [error, setError] = useState<AudioError | null>(null);

  const refresh = useCallback(() => {
    audioApi
      .listDevices()
      .then((list) => {
        setDevices(list);
        setError(null);
      })
      .catch((reason: AudioError) => setError(reason));
  }, []);

  useEffect(() => {
    refresh();
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
  }, [refresh]);

  return { devices, error, refresh };
}
