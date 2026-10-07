import "./LevelMeter.css";

interface LevelMeterProps {
  /** Loudness from 0 (silence) to 1 (full scale), already smoothed or not. */
  level: number;
  className?: string;
  "aria-label"?: string;
  "aria-labelledby"?: string;
}

/** Converts an RMS amplitude to a 0–1 meter value on a -60 to 0 dB scale. */
export function rmsToLevel(rms: number): number {
  if (rms <= 0) return 0;
  const db = 20 * Math.log10(rms);
  return Math.min(Math.max((db + 60) / 60, 0), 1);
}

/** A thin bar showing how loud the microphone is right now. */
export function LevelMeter({ level, className, ...aria }: LevelMeterProps) {
  const percent = Math.round(Math.min(Math.max(level, 0), 1) * 100);
  return (
    <div
      role="meter"
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={percent}
      className={["level-meter", className].filter(Boolean).join(" ")}
      {...aria}
    >
      <div className="level-meter__fill" style={{ width: `${percent}%` }} />
    </div>
  );
}
