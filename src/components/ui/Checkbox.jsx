import { Check } from 'lucide-react';

export function Checkbox({ checked, onChange, label }) {
  return (
    <label className="inline-flex items-center cursor-pointer">
      <button
        type="button"
        role="checkbox"
        aria-checked={checked}
        aria-label={label}
        onClick={() => onChange(!checked)}
        className={`w-4 h-4 rounded-[4px] border flex items-center justify-center transition-colors ${
          checked
            ? 'bg-[var(--fill-accent)] border-[var(--fill-accent)]'
            : 'bg-transparent border-[var(--border-strong)]'
        }`}
      >
        {checked && <Check size={12} className="text-[var(--on-accent)]" />}
      </button>
    </label>
  );
}
