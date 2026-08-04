export function SettingRow({ label, children, disabled = false }) {
  return (
    <div
      className={`flex items-center justify-between py-1.5 ${disabled ? 'opacity-50 pointer-events-none' : ''}`}
    >
      <span className="text-[13px] text-[var(--text-secondary)]">{label}</span>
      <div className="flex items-center gap-1.5">
        {children}
      </div>
    </div>
  );
}
