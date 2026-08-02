import { describe, it, expect } from 'vitest';
import { mergeSettings, DEFAULT_SETTINGS, settingsToConversionOptions, getEffectiveSettings } from './settings';

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
});

describe('settingsToConversionOptions', () => {
  it('maps settings to conversion options shape', () => {
    const result = settingsToConversionOptions(DEFAULT_SETTINGS, '/output');
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

  it('uses outputLocation.defaultFolder from settings when no explicit folder', () => {
    const settings = {
      ...DEFAULT_SETTINGS,
      outputLocation: { defaultFolder: '/custom/path' },
    };
    const result = settingsToConversionOptions(settings);
    expect(result.outputFolder).toBe('/custom/path');
  });

  it('prefers explicit outputFolder over settings default', () => {
    const settings = {
      ...DEFAULT_SETTINGS,
      outputLocation: { defaultFolder: '/custom/path' },
    };
    const result = settingsToConversionOptions(settings, '/explicit');
    expect(result.outputFolder).toBe('/explicit');
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
});
