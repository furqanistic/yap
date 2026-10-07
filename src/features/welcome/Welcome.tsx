import "./Welcome.css";
import { KeyboardKeys, type KeyDef } from "@/components/ui";
import { getPlatform } from "@/lib/platform";

interface WelcomeProps {
  /** Dismiss the welcome screen and enter the app. */
  onGetStarted: () => void;
  /** Jump to settings so the user can change the shortcut. */
  onCustomize: () => void;
}

/** Shortcut keys for the current platform (matches General settings default). */
function shortcutKeys(): KeyDef[] {
  if (getPlatform() === "macos") {
    return [
      { label: "⌥", sublabel: "option", size: "md", keyCode: "Alt" },
      { label: "Space", size: "xl", keyCode: " " },
    ];
  }
  return [
    { label: "Ctrl", size: "lg", keyCode: "Control" },
    { label: "Space", size: "xl", keyCode: " " },
  ];
}

export function Welcome({ onGetStarted, onCustomize }: WelcomeProps) {
  return (
    <div className="welcome">
      <div className="welcome__logo" aria-hidden="true">
        Y
      </div>

      <h1 className="welcome__title">Welcome to Yap</h1>
      <p className="welcome__subtitle">
        Speak anywhere on your screen — your words appear as you talk.
      </p>

      <KeyboardKeys
        keys={shortcutKeys()}
        label="Hold to talk · release to type"
        aria-label="Push-to-talk shortcut"
      />

      <div className="welcome__actions">
        <button
          type="button"
          className="welcome__button welcome__button--primary"
          onClick={onGetStarted}
        >
          Get Started
        </button>
        <button
          type="button"
          className="welcome__button welcome__button--secondary"
          onClick={onCustomize}
        >
          Customize Shortcut
        </button>
      </div>
    </div>
  );
}
