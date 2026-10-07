import { isTauri } from "@tauri-apps/api/core";

export type Platform = "macos" | "windows" | "linux";
export type WindowAppearance = "glass" | "solid";

const APPEARANCE_KEY = "yap.windowAppearance";

export function getPlatform(): Platform {
  const ua = navigator.userAgent;
  if (/Mac/i.test(ua)) return "macos";
  if (/Win/i.test(ua)) return "windows";
  return "linux";
}

/**
 * Platforms where the OS reliably composites a blur behind the window.
 * Linux window effects vary across DEs and display servers (and are often
 * missing entirely), so Yap never relies on them — see applyPlatformAttributes.
 */
function supportsNativeBlur(platform: Platform): boolean {
  return platform === "macos" || platform === "windows";
}

/** The user's window appearance preference. Defaults to glass where supported. */
export function getWindowAppearance(): WindowAppearance {
  try {
    const value = localStorage.getItem(APPEARANCE_KEY);
    return value === "solid" ? "solid" : "glass";
  } catch {
    return "glass";
  }
}

/** Persists the appearance choice and reapplies window attributes immediately. */
export function setWindowAppearance(appearance: WindowAppearance) {
  try {
    localStorage.setItem(APPEARANCE_KEY, appearance);
  } catch {
    // Storage unavailable (e.g. dev tools restrictions); keep going — the
    // choice just won't survive a restart.
  }
  applyPlatformAttributes();
}

/** True when the OS blur is actually active right now. */
export function isGlassActive(): boolean {
  return document.documentElement.dataset.vibrancy === "native";
}

/** True when the app draws its own window buttons (Windows/Linux inside Tauri). */
export function usesCustomWindowControls(): boolean {
  return isTauri() && getPlatform() !== "macos";
}

/**
 * Marks the document with the current platform and, inside the Tauri window,
 * native translucency (Acrylic on Windows, vibrancy on macOS). Styles use
 * these to let the desktop show through and to make room for macOS traffic
 * lights.
 *
 * Translucency is intentionally restricted:
 * - macOS and Windows only — both composite a real blur behind the window.
 * - Linux never gets `data-vibrancy`, so tokens stay fully opaque and text
 *   remains readable over any wallpaper (issue #8).
 * - Users can force solid windows on every platform via Settings → General →
 *   Window appearance.
 */
export function applyPlatformAttributes() {
  const root = document.documentElement;
  const platform = getPlatform();
  root.dataset.platform = platform;

  const glass =
    isTauri() && supportsNativeBlur(platform) && getWindowAppearance() === "glass";

  if (glass) {
    root.dataset.vibrancy = "native";
  } else {
    delete root.dataset.vibrancy;
  }
}
