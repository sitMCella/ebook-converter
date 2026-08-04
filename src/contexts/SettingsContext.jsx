import { createContext, useContext, useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { toast } from 'sonner';
import { DEFAULT_SETTINGS, loadSettings, saveSettings } from '../lib/settings';

const SettingsContext = createContext(null);

function debounce(fn, ms) {
  let timer;
  const debounced = (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
  debounced.flush = () => {
    clearTimeout(timer);
  };
  return debounced;
}

export function SettingsProvider({ children }) {
  const [settings, setSettings] = useState(DEFAULT_SETTINGS);
  const [loaded, setLoaded] = useState(false);
  const settingsRef = useRef(settings);

  const debouncedSave = useMemo(
    () => debounce((s) => saveSettings(s), 300),
    [],
  );

  useEffect(() => {
    loadSettings().then((s) => {
      setSettings(s);
      settingsRef.current = s;
      setLoaded(true);
    });
  }, []);

  const updateSetting = useCallback((group, key, value) => {
    setSettings(prev => {
      const next = {
        ...prev,
        [group]: { ...prev[group], [key]: value },
      };

      if (group === 'images' && key === 'convertToWebP' && value === true) {
        if (next.output.epubVersion === 'epub2') {
          next.output = { ...next.output, epubVersion: 'epub3' };
          toast.warning('WebP images require EPUB 3. The EPUB version setting will be changed to EPUB 3.');
        }
      }

      if (group === 'output' && key === 'epubVersion' && value === 'epub2') {
        if (next.images.convertToWebP) {
          next.images = { ...next.images, convertToWebP: false };
          toast.warning('WebP images are not supported in EPUB 2. Image conversion has been disabled.');
        }
      }

      if (group === 'structure' && key === 'detectHeadings' && value === false) {
        if (next.pageHandling.splitChaptersBy === 'heading1' || next.pageHandling.splitChaptersBy === 'heading2') {
          toast.warning('Chapter splitting by headings requires heading detection. Consider changing the split strategy.');
        }
      }

      settingsRef.current = next;
      debouncedSave(next);
      return next;
    });
  }, [debouncedSave]);

  const resetToDefaults = useCallback(async () => {
    const defaults = { ...DEFAULT_SETTINGS };
    setSettings(defaults);
    settingsRef.current = defaults;
    await saveSettings(defaults);
  }, []);

  const value = useMemo(
    () => ({ settings, loaded, updateSetting, resetToDefaults }),
    [settings, loaded, updateSetting, resetToDefaults],
  );

  return (
    <SettingsContext.Provider value={value}>
      {children}
    </SettingsContext.Provider>
  );
}

export function useSettings() {
  const context = useContext(SettingsContext);
  if (!context) {
    throw new Error('useSettings must be used within a SettingsProvider');
  }
  return context;
}
