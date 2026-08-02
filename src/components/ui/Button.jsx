export function Button({ variant = 'primary', disabled = false, onClick, children, ...props }) {
  const base =
    'inline-flex items-center gap-2 px-4 py-2 text-[13px] font-medium rounded-[var(--radius)] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed';

  const variants = {
    primary:
      'bg-[var(--fill-accent)] text-[var(--on-accent)] hover:opacity-90',
    secondary:
      'bg-transparent border border-[var(--border-strong)] text-[var(--text-secondary)] hover:bg-[var(--surface-2)]',
  };

  return (
    <button
      type="button"
      className={`${base} ${variants[variant]}`}
      disabled={disabled}
      onClick={onClick}
      {...props}
    >
      {children}
    </button>
  );
}
