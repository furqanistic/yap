/** Mirrors `src-tauri/src/hotkey`. */
export interface HotkeyStatus {
  /** True when the keyboard hook is running. */
  installed: boolean;
  /** The active chord, as stored in settings. */
  chord: string[];
  /** Why hold-to-talk isn't working, if it isn't. */
  error: string | null;
}
