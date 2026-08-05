import { describe, it, expect } from 'vitest';
import { mergeSettings, DEFAULT_SETTINGS, settingsToConversionOptions, getEffectiveSettings } from './settings';

describe('DEFAULT_SETTINGS', () => {
  it('has textAlignment in output group', () => {
    expect(DEFAULT_SETTINGS.output.textAlignment).toBe('justify');
  });

  it('has keepPageBreaks in pageHandling group', () => {
    expect(DEFAULT_SETTINGS.pageHandling.keepPageBreaks).toBe(false);
  });

  it('has removePageNumbers in pageHandling group', () => {
    expect(DEFAULT_SETTINGS.pageHandling.removePageNumbers).toBe(true);
  });

  it('has coverPage in pageHandling group', () => {
    expect(DEFAULT_SETTINGS.pageHandling.coverPage).toBe('auto');
  });

  it('does not have outputLocation', () => {
    expect(DEFAULT_SETTINGS.outputLocation).toBeUndefined();
  });
});

describe('mergeSettings', () => {
  it('returns base when no overrides', () => {
    const result = mergeSettings(DEFAULT_SETTINGS, {});
    expect(result.structure.detectHeadings).toBe(true);
    expect(result.output.epubVersion).toBe('epub3');
  });

  it('overrides nested values', () => {
    const result = mergeSettings(DEFAULT_SETTINGS, {
      structure: { detectHeadings: false },
    });
    expect(result.structure.detectHeadings).toBe(false);
    expect(result.structure.paragraphDetection).toBe(true);
  });

  it('overrides top-level values', () => {
    const result = mergeSettings(DEFAULT_SETTINGS, {
      images: { imageQuality: 'high', maxImageWidth: 1200 },
    });
    expect(result.images.imageQuality).toBe('high');
    expect(result.images.maxImageWidth).toBe(1200);
    expect(result.images.extractImages).toBe(true);
  });

  it('handles null overrides', () => {
    const result = mergeSettings(DEFAULT_SETTINGS, null);
    expect(result.structure.detectHeadings).toBe(true);
  });

  it('fills in missing new fields from old settings file', () => {
    const oldSettings = {
      structure: { detectHeadings: false },
      output: { epubVersion: 'epub2' },
      pageHandling: { skipBlankPages: false },
    };
    const result = mergeSettings(DEFAULT_SETTINGS, oldSettings);
    expect(result.output.textAlignment).toBe('justify');
    expect(result.pageHandling.keepPageBreaks).toBe(false);
    expect(result.pageHandling.removePageNumbers).toBe(true);
    expect(result.pageHandling.coverPage).toBe('auto');
    expect(result.structure.detectHeadings).toBe(false);
    expect(result.output.epubVersion).toBe('epub2');
    expect(result.pageHandling.skipBlankPages).toBe(false);
  });
});

describe('settingsToConversionOptions', () => {
  it('maps settings to conversion options shape', () => {
    const result = settingsToConversionOptions(DEFAULT_SETTINGS, { outputFolder: '/output' });
    expect(result.outputFolder).toBe('/output');
    expect(result.structure).toBe(DEFAULT_SETTINGS.structure);
    expect(result.images).toBe(DEFAULT_SETTINGS.images);
    expect(result.output).toBe(DEFAULT_SETTINGS.output);
    expect(result.pageHandling).toBe(DEFAULT_SETTINGS.pageHandling);
  });

  it('uses default folder when none provided', () => {
    const result = settingsToConversionOptions(DEFAULT_SETTINGS);
    expect(result.outputFolder).toBe('~/Documents/Ebooks');
  });

  it('prefers explicit outputFolder over default', () => {
    const result = settingsToConversionOptions(DEFAULT_SETTINGS, { outputFolder: '/explicit' });
    expect(result.outputFolder).toBe('/explicit');
  });

  it('passes bookId through to options', () => {
    const result = settingsToConversionOptions(DEFAULT_SETTINGS, { bookId: 'abc-123' });
    expect(result.bookId).toBe('abc-123');
  });

  it('defaults bookId to null when not provided', () => {
    const result = settingsToConversionOptions(DEFAULT_SETTINGS);
    expect(result.bookId).toBeNull();
  });
});

describe('getEffectiveSettings', () => {
  it('returns global settings when no overrides', () => {
    const result = getEffectiveSettings(DEFAULT_SETTINGS, {});
    expect(result.structure.detectHeadings).toBe(true);
  });

  it('merges document overrides into global settings', () => {
    const result = getEffectiveSettings(DEFAULT_SETTINGS, {
      images: { imageQuality: 'high' },
    });
    expect(result.images.imageQuality).toBe('high');
    expect(result.images.extractImages).toBe(true);
  });

  it('handles null overrides', () => {
    const result = getEffectiveSettings(DEFAULT_SETTINGS, null);
    expect(result.output.epubVersion).toBe('epub3');
  });

  it('overrides coverPage per document', () => {
    const result = getEffectiveSettings(DEFAULT_SETTINGS, {
      pageHandling: { coverPage: 'none' },
    });
    expect(result.pageHandling.coverPage).toBe('none');
    expect(result.pageHandling.skipBlankPages).toBe(true);
  });

  it('preserves coverPage default when not overridden', () => {
    const result = getEffectiveSettings(DEFAULT_SETTINGS, {
      pageHandling: { skipBlankPages: false },
    });
    expect(result.pageHandling.coverPage).toBe('auto');
  });
});

describe('settingsToConversionOptions coverPage', () => {
  it('passes coverPage through to conversion options', () => {
    const settings = {
      ...DEFAULT_SETTINGS,
      pageHandling: { ...DEFAULT_SETTINGS.pageHandling, coverPage: 'firstPage' },
    };
    const result = settingsToConversionOptions(settings);
    expect(result.pageHandling.coverPage).toBe('firstPage');
  });

  it('passes coverPage=none through to conversion options', () => {
    const settings = {
      ...DEFAULT_SETTINGS,
      pageHandling: { ...DEFAULT_SETTINGS.pageHandling, coverPage: 'none' },
    };
    const result = settingsToConversionOptions(settings);
    expect(result.pageHandling.coverPage).toBe('none');
  });
});
