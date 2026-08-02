import { useEffect, useRef } from 'react';
import { Info } from 'lucide-react';
import { useConversionContext } from '../../contexts/ConversionContext';

export function ConversionLog() {
  const { state } = useConversionContext();
  const endRef = useRef(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [state.logEntries.length]);

  return (
    <div className="border border-[var(--border)] rounded-[var(--radius)] overflow-hidden">
      <div className="flex items-center gap-2 px-3.5 py-2 border-b border-[var(--border)]">
        <Info size={14} className="text-[var(--text-muted)]" />
        <span className="text-[13px] font-medium text-[var(--text-primary)]">
          Conversion log
        </span>
      </div>
      <div
        className="max-h-48 overflow-y-auto p-3"
        aria-live="polite"
        aria-label="Conversion log entries"
      >
        {state.logEntries.length === 0 && (
          <p className="text-[12px] font-mono text-[var(--text-muted)]">
            Waiting for conversion to start...
          </p>
        )}
        {state.logEntries.map((entry, i) => (
          <p
            key={i}
            className={`text-[12px] font-mono leading-5 ${
              entry.level === 'error'
                ? 'text-[var(--text-danger)]'
                : 'text-[var(--text-muted)]'
            }`}
          >
            {entry.message}
          </p>
        ))}
        <div ref={endRef} />
      </div>
    </div>
  );
}
