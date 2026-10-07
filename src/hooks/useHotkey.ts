import { useEffect, useState } from "react";
import { hotkeyApi, onEvent } from "@/lib/tauri";
import type { HotkeyStatus } from "@/types/hotkey";

export interface UseHotkey {
  /** True while the hold-to-talk shortcut is held. */
  pressed: boolean;
  status: HotkeyStatus | null;
}

/** Live hold-to-talk state, and whether the global shortcut works at all. */
export function useHotkey(chord: string[]): UseHotkey {
  const [pressed, setPressed] = useState(false);
  const [status, setStatus] = useState<HotkeyStatus | null>(null);
  const chordKey = chord.join("+");

  useEffect(() => {
    const stops = [
      onEvent("hotkey://down", () => setPressed(true)),
      onEvent("hotkey://up", () => setPressed(false)),
      onEvent("hotkey://cancelled", () => setPressed(false)),
    ];
    return () => stops.forEach((stop) => stop());
  }, []);

  // Re-read the status when the chord changes, since a new chord can be invalid.
  useEffect(() => {
    hotkeyApi.status().then(setStatus).catch(() => setStatus(null));
  }, [chordKey]);

  return { pressed, status };
}
