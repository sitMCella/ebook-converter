export function ProgressBar({ percent = 0, label }) {
  return (
    <div
      role="progressbar"
      aria-valuenow={percent}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-label={label || `Conversion progress: ${percent}%`}
      className="w-full h-1 rounded-sm bg-[var(--border)] overflow-hidden"
    >
      <div
        className="h-full bg-[var(--fill-accent)] rounded-sm transition-all duration-300"
        style={{ width: `${Math.min(100, Math.max(0, percent))}%` }}
      />
    </div>
  );
}
