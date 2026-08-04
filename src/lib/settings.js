import { isTauri } from './tauri';

export const DEFAULT_SETTINGS = {
  structure: {
    detectHeadings: true,
    detectToc: true,
    detectFootnotes: false,
    headingLevelThreshold: 3,
    paragraphDetection: true,
    listDetection: true,
  },
  images: {
    extractImages: true,
    imageQuality: 'medium',
    maxImageWidth: 800,
    convertToWebP: false,
  },
  output: {
    epubVersion: 'epub3',
    embedFonts: false,
    fontFamily: 'default',
    baseFontSize: 12,
    lineHeight: 1.5,
    margins: 1.0,
    textAlignment: 'justify',
  },
  pageHandling: {
    skipBlankPages: true,
    pageRange: 'all',
    pageRangeFrom: null,
    pageRangeTo: null,
    splitChaptersBy: 'heading1',
    keepPageBreaks: false,
    removePageNumbers: true,
    coverPage: 'auto',
  },
};

export async function loadSettings() {
  if (!isTauri) return { ...DEFAULT_SETTINGS };

  try {
    const { appDataDir } = await import('@tauri-apps/api/path');
    const { readTextFile, exists } = await import('@tauri-apps/plugin-fs');

    const dir = await appDataDir();
    const path = `${dir}settings.json`;

    if (!(await exists(path))) {
      return { ...DEFAULT_SETTINGS };
    }

    const text = await readTextFile(path);
    const parsed = JSON.parse(text);
    return mergeSettings(DEFAULT_SETTINGS, parsed);
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

export async function saveSettings(settings) {
  if (!isTauri) return;

  try {
    const { appDataDir } = await import('@tauri-apps/api/path');
    const { writeTextFile, mkdir, exists } = await import('@tauri-apps/plugin-fs');

    const dir = await appDataDir();
    if (!(await exists(dir))) {
      await mkdir(dir, { recursive: true });
    }

    const path = `${dir}settings.json`;
    await writeTextFile(path, JSON.stringify(settings, null, 2));
  } catch (e) {
    console.error('Failed to save settings:', e);
  }
}

export function mergeSettings(base, overrides) {
  const result = {};
  for (const key of Object.keys(base)) {
    if (
      typeof base[key] === 'object' &&
      base[key] !== null &&
      !Array.isArray(base[key]) &&
      overrides &&
      typeof overrides[key] === 'object' &&
      overrides[key] !== null
    ) {
      result[key] = { ...base[key], ...overrides[key] };
    } else if (overrides && key in overrides) {
      result[key] = overrides[key];
    } else {
      result[key] = base[key];
    }
  }
  return result;
}

export function getEffectiveSettings(globalSettings, documentOverrides) {
  return mergeSettings(globalSettings, documentOverrides);
}

export function settingsToConversionOptions(settings, { outputFolder, bookId } = {}) {
  return {
    structure: settings.structure,
    images: settings.images,
    output: settings.output,
    pageHandling: settings.pageHandling,
    outputFolder: outputFolder || '~/Documents/Ebooks',
    bookId: bookId || null,
  };
}
