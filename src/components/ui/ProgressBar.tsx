import "./ProgressBar.css";

interface ProgressBarProps {
  /** Current progress. Leave undefined for an indeterminate bar. */
  value?: number;
  max?: number;
  className?: string;
  "aria-label"?: string;
  "aria-labelledby"?: string;
  "aria-describedby"?: string;
}

export function ProgressBar({ value, max = 100, className, ...aria }: ProgressBarProps) {
  const indeterminate = value === undefined;
  const percent = indeterminate || max <= 0 ? 0 : Math.min(Math.max(value / max, 0), 1) * 100;

  return (
    <div
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={max}
      aria-valuenow={indeterminate ? undefined : value}
      className={["progress", indeterminate && "progress--indeterminate", className].filter(Boolean).join(" ")}
      {...aria}
    >
      <div className="progress__fill" style={indeterminate ? undefined : { width: `${percent}%` }} />
    </div>
  );
}
