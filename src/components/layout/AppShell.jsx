import { Outlet } from 'react-router-dom';
import { Sidebar } from './Sidebar';

export function AppShell() {
  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar />
      <main className="flex-1 overflow-auto bg-[var(--surface-1)]">
        <div className="p-[20px_24px]">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
