/** Mirrors `src-tauri/src/models`. */
import type { TranscriptionModel } from "./settings";

export type ModelId = TranscriptionModel;

export type ModelLanguages = "englishOnly" | "multilingual";

export type ModelErrorKind =
  | "offline"
  | "diskFull"
  | "httpStatus"
  | "checksumMismatch"
  | "cancelled"
  | "io"
  | "busy"
  | "unknownModel"
  | "notAllowed";

export interface ModelError {
  kind: ModelErrorKind;
  /** Plain-language message, ready to show. */
  message: string;
}

export type ModelState =
  /** `partialBytes` > 0 means an interrupted download can resume. */
  | { status: "notDownloaded"; partialBytes: number }
  | { status: "downloading"; downloaded: number; total: number }
  | { status: "verifying" }
  | { status: "ready" }
  | { status: "failed"; error: ModelError };

export interface ModelEntry {
  id: ModelId;
  label: string;
  description: string;
  languages: ModelLanguages;
  recommended: boolean;
  sizeBytes: number;
  url: string;
  state: ModelState;
}

export interface ModelProgressEvent {
  id: ModelId;
  downloaded: number;
  total: number;
  bytesPerSec: number;
}

export interface DiskSpace {
  availableBytes: number;
  /** What's still to download, plus a safety margin. */
  requiredBytes: number;
}

export interface ModelStateEvent {
  id: ModelId;
  state: ModelState;
}
