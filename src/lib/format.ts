/** Human-friendly formatting for sizes, speeds and durations. */

const KB = 1024;
const MB = KB * 1024;
const GB = MB * 1024;

/**
 * formatBytes(0) → "0 MB"
 * formatBytes(77_691_713) → "74 MB"
 * formatBytes(1_288_490_189) → "1.2 GB"
 */
export function formatBytes(bytes: number): string {
  if (bytes >= GB) return `${(bytes / GB).toFixed(1)} GB`;
  return `${Math.round(bytes / MB)} MB`;
}

/**
 * formatSpeed(4_404_019) → "4.2 MB/s"
 * formatSpeed(512_000) → "500 KB/s"
 */
export function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec >= MB) return `${(bytesPerSec / MB).toFixed(1)} MB/s`;
  return `${Math.max(1, Math.round(bytesPerSec / KB))} KB/s`;
}

/**
 * Rough time left, rounded so it doesn't jitter.
 * formatEta(8) → "a few seconds left"
 * formatEta(42) → "less than a minute left"
 * formatEta(150) → "about 3 min left"
 * formatEta(5400) → "about 1 hr 30 min left"
 */
export function formatEta(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return "";
  if (seconds < 15) return "a few seconds left";
  if (seconds < 60) return "less than a minute left";
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `about ${minutes} min left`;
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  return rest === 0 ? `about ${hours} hr left` : `about ${hours} hr ${rest} min left`;
}

/** formatPercent(412, 466) → "88%" */
export function formatPercent(value: number, total: number): string {
  if (total <= 0) return "0%";
  return `${Math.floor((value / total) * 100)}%`;
}
