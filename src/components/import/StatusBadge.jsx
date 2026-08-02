const variants = {
  ready: 'bg-[var(--bg-accent)] text-[var(--text-accent)]',
  converting: 'bg-[var(--bg-warning)] text-[var(--text-warning)]',
  converted: 'bg-[var(--bg-success)] text-[var(--text-success)]',
  error: 'bg-[var(--bg-danger)] text-[var(--text-danger)]',
};

const labels = {
  ready: 'Ready',
  converting: 'Converting',
  converted: 'Converted',
  error: 'Error',
};

export function StatusBadge({ status }) {
  return (
    <span
      className={`inline-block px-2.5 py-0.5 text-[12px] rounded-full ${variants[status]}`}
      aria-label={`Status: ${labels[status]}`}
    >
      {labels[status]}
    </span>
  );
}
