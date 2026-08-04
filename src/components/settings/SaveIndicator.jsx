import { useState, useEffect } from 'react';
import { Check } from 'lucide-react';

export function SaveIndicator({ visible }) {
  const [show, setShow] = useState(false);

  useEffect(() => {
    if (visible) {
      setShow(true);
      const timer = setTimeout(() => setShow(false), 1000);
      return () => clearTimeout(timer);
    }
  }, [visible]);

  if (!show) return null;

  return (
    <span
      className="inline-flex transition-opacity duration-500"
      style={{ opacity: show ? 1 : 0 }}
    >
      <Check size={14} className="text-[var(--text-success)]" />
    </span>
  );
}
