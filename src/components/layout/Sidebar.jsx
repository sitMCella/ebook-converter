import { NavLink } from 'react-router-dom';
import { Upload, FileText, Book, Settings } from 'lucide-react';

const navItems = [
  { to: '/import', icon: Upload, label: 'Import' },
  { to: '/library', icon: FileText, label: 'Library' },
  { to: '/converted', icon: Book, label: 'Converted' },
];

export function Sidebar() {
  return (
    <nav className="w-[200px] min-w-[200px] h-full bg-[var(--surface-0)] border-r border-[var(--border)] flex flex-col">
      <div className="flex-1 py-2">
        {navItems.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              `flex items-center gap-3 px-4 py-2 text-[13px] transition-colors ${
                isActive
                  ? 'bg-[var(--bg-accent)] text-[var(--text-accent)] font-medium'
                  : 'text-[var(--text-secondary)] hover:bg-[var(--surface-2)]'
              }`
            }
          >
            <Icon size={18} />
            {label}
          </NavLink>
        ))}

        <div className="mx-4 my-2 border-t border-[var(--border)]" />
        <div className="px-4 py-1 text-[11px] text-[var(--text-muted)] uppercase tracking-wider">
          Tools
        </div>

        <NavLink
          to="/settings"
          className={({ isActive }) =>
            `flex items-center gap-3 px-4 py-2 text-[13px] transition-colors ${
              isActive
                ? 'bg-[var(--bg-accent)] text-[var(--text-accent)] font-medium'
                : 'text-[var(--text-secondary)] hover:bg-[var(--surface-2)]'
            }`
          }
        >
          <Settings size={18} />
          Settings
        </NavLink>
      </div>
    </nav>
  );
}
