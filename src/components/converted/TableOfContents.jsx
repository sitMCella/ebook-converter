import { useState } from 'react';
import { ChevronRight, ChevronDown } from 'lucide-react';

export function TableOfContents() {
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
      </button>
      {expanded && (
        <div className="mt-2 pl-6">
          <p className="text-[12px] text-[var(--text-muted)]">
            Table of contents not yet available.
          </p>
        </div>
      )}
    </div>
  );
}
