import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { Toaster } from 'sonner';
import { ImportProvider } from './contexts/ImportContext';
import { ConversionProvider } from './contexts/ConversionContext';
import { AppShell } from './components/layout/AppShell';
import { ImportScreen } from './components/import/ImportScreen';
import { ConvertingScreen } from './components/conversion/ConvertingScreen';
import { LibraryScreen } from './components/library/LibraryScreen';
import { ConvertedScreen } from './components/converted/ConvertedScreen';

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
      <ConversionProvider>
        <BrowserRouter>
          <Routes>
            <Route element={<AppShell />}>
              <Route index element={<Navigate to="/import" replace />} />
              <Route path="import" element={<ImportScreen />} />
              <Route path="converting" element={<ConvertingScreen />} />
              <Route path="library" element={<LibraryScreen />} />
              <Route path="converted" element={<ConvertedScreen />} />
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
      </ConversionProvider>
    </ImportProvider>
  );
}

export default App;
