import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ProgressBar } from './ProgressBar';

describe('ProgressBar', () => {
  it('renders with correct aria attributes', () => {
    render(<ProgressBar percent={42} />);
    const bar = screen.getByRole('progressbar');
    expect(bar).toHaveAttribute('aria-valuenow', '42');
    expect(bar).toHaveAttribute('aria-valuemin', '0');
    expect(bar).toHaveAttribute('aria-valuemax', '100');
  });

  it('uses custom label', () => {
    render(<ProgressBar percent={50} label="Converting test.pdf" />);
    const bar = screen.getByRole('progressbar');
    expect(bar).toHaveAttribute('aria-label', 'Converting test.pdf');
  });

  it('defaults percent to 0', () => {
    render(<ProgressBar />);
    const bar = screen.getByRole('progressbar');
    expect(bar).toHaveAttribute('aria-valuenow', '0');
  });

  it('clamps width to 0-100 range', () => {
    const { container } = render(<ProgressBar percent={150} />);
    const fill = container.querySelector('[role="progressbar"] > div');
    expect(fill.style.width).toBe('100%');
  });
});
