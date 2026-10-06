import { isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

/**
 * Opens an external URL in the user's default browser.
 * Inside Tauri, uses the native opener plugin; in a standard browser, falls back to window.open.
 */
export async function openExternalUrl(url: string): Promise<void> {
  if (isTauri()) {
    try {
      await openUrl(url);
      return;
    } catch (error) {
      console.error("Failed to open URL via Tauri plugin opener:", error);
    }
  }

  window.open(url, "_blank", "noopener,noreferrer");
}
