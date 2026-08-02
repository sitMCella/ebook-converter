import { describe, it, expect } from 'vitest';
import { formatFileSize } from './format';

describe('formatFileSize', () => {
  it('formats 0 bytes', () => {
    expect(formatFileSize(0)).toBe('0 B');
  });

  it('formats bytes below 1024', () => {
    expect(formatFileSize(1)).toBe('1 B');
    expect(formatFileSize(512)).toBe('512 B');
    expect(formatFileSize(1023)).toBe('1023 B');
  });

  it('formats kilobytes', () => {
    expect(formatFileSize(1024)).toBe('1 KB');
    expect(formatFileSize(2048)).toBe('2 KB');
    expect(formatFileSize(1536)).toBe('2 KB');
  });

  it('formats megabytes', () => {
    expect(formatFileSize(1024 * 1024)).toBe('1.0 MB');
    expect(formatFileSize(1024 * 1024 * 5.5)).toBe('5.5 MB');
    expect(formatFileSize(1024 * 1024 * 100)).toBe('100.0 MB');
  });

  it('formats gigabytes', () => {
    expect(formatFileSize(1024 * 1024 * 1024)).toBe('1.00 GB');
    expect(formatFileSize(1024 * 1024 * 1024 * 2.5)).toBe('2.50 GB');
  });

  it('handles the KB/MB boundary', () => {
    expect(formatFileSize(1024 * 999)).toBe('999 KB');
    expect(formatFileSize(1024 * 1000)).toBe('1.0 MB');
  });

  it('handles the MB/GB boundary', () => {
    expect(formatFileSize(1024 * 1000 * 999)).toBe('975.6 MB');
    expect(formatFileSize(1024 * 1000 * 1000)).toBe('0.95 GB');
  });
});
