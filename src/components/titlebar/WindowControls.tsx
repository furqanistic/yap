import { getCurrentWindow } from "@tauri-apps/api/window";
import { useWindowMaximized } from "@/hooks/useWindowMaximized";
import { LiquidGlassSurface } from "@/components/liquid-glass";

/** Windows 11-style minimize / maximize / close buttons. */
export function WindowControls() {
  const appWindow = getCurrentWindow();
  const maximized = useWindowMaximized();

  return (
    <div className="window-controls">
      <LiquidGlassSurface
        as="button"
        type="button"
        preset="interactive"
        refraction={false}
        className="window-controls__button"
        aria-label="Minimize"
        onClick={() => appWindow.minimize()}
      >
        <svg viewBox="0 0 10 10" aria-hidden="true">
          <path d="M0 5.5h10" />
        </svg>
      </LiquidGlassSurface>

      <LiquidGlassSurface
        as="button"
        type="button"
        preset="interactive"
        refraction={false}
        className="window-controls__button"
        aria-label={maximized ? "Restore" : "Maximize"}
        onClick={() => appWindow.toggleMaximize()}
      >
        <svg viewBox="0 0 10 10" aria-hidden="true">
          {maximized ? (
            <path d="M0.5 2.5h7v7h-7z M2.5 2.5v-2h7v7h-2" />
          ) : (
            <path d="M0.5 0.5h9v9h-9z" />
          )}
        </svg>
      </LiquidGlassSurface>

      <LiquidGlassSurface
        as="button"
        type="button"
        preset="interactive"
        refraction={false}
        className="window-controls__button window-controls__button--close"
        aria-label="Close"
        onClick={() => appWindow.close()}
      >
        <svg viewBox="0 0 10 10" aria-hidden="true">
          <path d="M0.5 0.5l9 9M9.5 0.5l-9 9" />
        </svg>
      </LiquidGlassSurface>
    </div>
  );
}
