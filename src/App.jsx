import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { Toaster } from 'sonner';
import { ImportProvider } from './contexts/ImportContext';
import { AppShell } from './components/layout/AppShell';
import { ImportScreen } from './components/import/ImportScreen';

function Placeholder({ title }) {
  return (
    <div className="flex items-center justify-center h-64 text-[var(--text-muted)]">
      <p>{title} — coming soon</p>
    </div>
  );
}

function App() {
  return (
    <ImportProvider>
      <BrowserRouter>
        <Routes>
          <Route element={<AppShell />}>
            <Route index element={<Navigate to="/import" replace />} />
            <Route path="import" element={<ImportScreen />} />
            <Route path="library" element={<Placeholder title="Library" />} />
            <Route path="converted" element={<Placeholder title="Converted" />} />
            <Route path="settings" element={<Placeholder title="Settings" />} />
          </Route>
        </Routes>
      </BrowserRouter>
      <Toaster
        position="bottom-center"
        visibleToasts={3}
        duration={3000}
        closeButton
      />
    </ImportProvider>
  );
}

export default App;
