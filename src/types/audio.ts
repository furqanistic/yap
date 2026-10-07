/** Mirrors `src-tauri/src/audio`. */

export interface InputDevice {
  /** Stable id stored in settings (`microphone`). */
  id: string;
  name: string;
  isDefault: boolean;
}

export type AudioErrorKind = "noDevice" | "permissionDenied" | "deviceBusy" | "unsupported" | "other";

export interface AudioError {
  kind: AudioErrorKind;
  /** Plain-language message, ready to show. */
  message: string;
}

export interface AudioLevel {
  /** Root mean square of the last ~33 ms, 0 to 1. */
  rms: number;
  peak: number;
}

export interface MicTestEnded {
  reason: "deviceLost" | "timeLimit";
  message: string;
}
