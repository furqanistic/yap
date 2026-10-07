/** Mirrors `src-tauri/src/stt`. */

export interface SttStatus {
  /** The model that's loaded and ready. */
  model: string | null;
  /** The model being loaded right now. */
  loading: string | null;
  error: string | null;
}
