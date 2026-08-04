import { useState } from 'react';
import { ChevronRight, ChevronDown } from 'lucide-react';

export function TableOfContents({ entries = [] }) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div>
      <button
        type="button"
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-2 text-[14px] font-medium w-full text-left py-1 cursor-pointer text-[var(--text-primary)]"
      >
        {expanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
        Table of contents
        {entries.length > 0 && (
          <span className="text-[12px] text-[var(--text-muted)] font-normal">
            ({entries.length})
          </span>
        )}
      </button>
      {expanded && (
        <div className="mt-2 pl-6">
          {entries.length === 0 ? (
            <p className="text-[12px] text-[var(--text-muted)]">
              No chapters detected.
            </p>
          ) : (
            <ol className="list-none flex flex-col gap-1">
              {entries.map((entry, index) => (
                <li
                  key={index}
                  className="text-[13px] text-[var(--text-primary)] leading-snug"
                  style={{ paddingLeft: `${(entry.level - 1) * 16}px` }}
                >
                  <span className="text-[var(--text-muted)] mr-2 text-[12px] tabular-nums">
                    {index + 1}.
                  </span>
                  {entry.title}
                </li>
              ))}
            </ol>
          )}
        </div>
      )}
    </div>
  );
}
