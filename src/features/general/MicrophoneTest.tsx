import { useEffect, useRef, useState } from "react";
import type { SettingControlA11y } from "@/components/settings";
import { Button, LevelMeter, rmsToLevel } from "@/components/ui";
import { getPlatform } from "@/lib/platform";
import { audioApi, onEvent } from "@/lib/tauri";
import type { AudioError } from "@/types/audio";

interface MicrophoneTestProps extends SettingControlA11y {
  deviceId: string;
}

/** How fast the meter falls back after a loud moment (per update, ~30/s). */
const DECAY = 0.82;

/** Start/Stop button with a live level meter for the chosen microphone. */
export function MicrophoneTest({ deviceId, ...a11y }: MicrophoneTestProps) {
  const [running, setRunning] = useState(false);
  const [starting, setStarting] = useState(false);
  const [level, setLevel] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [needsPermission, setNeedsPermission] = useState(false);
  const levelRef = useRef(0);

  useEffect(() => {
    if (!running) return;
    const stopLevels = onEvent("audio://level", ({ rms }) => {
      levelRef.current = Math.max(rmsToLevel(rms), levelRef.current * DECAY);
      setLevel(levelRef.current);
    });
    const stopEnded = onEvent("audio://test-ended", ({ message }) => {
      setRunning(false);
      setError(message);
    });
    return () => {
      stopLevels();
      stopEnded();
    };
  }, [running]);

  // Stop when leaving the page or switching microphones.
  useEffect(() => {
    return () => {
      audioApi.testStop();
    };
  }, [deviceId]);

  useEffect(() => {
    if (running) return;
    levelRef.current = 0;
    setLevel(0);
  }, [running]);

  const start = async () => {
    setError(null);
    setNeedsPermission(false);
    setStarting(true);
    try {
      await audioApi.testStart(deviceId);
      setRunning(true);
    } catch (reason) {
      const audioError = reason as AudioError;
      setError(audioError.message);
      setNeedsPermission(audioError.kind === "permissionDenied" || audioError.kind === "noDevice");
    } finally {
      setStarting(false);
    }
  };

  const stop = () => {
    audioApi.testStop();
    setRunning(false);
  };

  const canOpenSettings = needsPermission && getPlatform() !== "linux";

  return (
    <div className="mic-test">
      <div className="mic-test__controls">
        <LevelMeter className="mic-test__meter" level={level} aria-label="Microphone level" />
        <Button size="small" loading={starting} onClick={running ? stop : start} {...a11y}>
          {running ? "Stop" : "Start"}
        </Button>
      </div>
      {error && (
        <p className="mic-test__error" role="alert">
          {error}
        </p>
      )}
      {canOpenSettings && (
        <Button size="small" variant="quiet" onClick={() => audioApi.openPrivacySettings()}>
          Open privacy settings
        </Button>
      )}
    </div>
  );
}
