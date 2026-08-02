import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { StatusBadge } from './StatusBadge';

describe('StatusBadge', () => {
  it('renders Ready badge', () => {
    render(<StatusBadge status="ready" />);
    expect(screen.getByText('Ready')).toBeInTheDocument();
  });

  it('renders Converting badge', () => {
    render(<StatusBadge status="converting" />);
    expect(screen.getByText('Converting')).toBeInTheDocument();
  });

  it('renders Converted badge', () => {
    render(<StatusBadge status="converted" />);
    expect(screen.getByText('Converted')).toBeInTheDocument();
  });

  it('renders Error badge', () => {
    render(<StatusBadge status="error" />);
    expect(screen.getByText('Error')).toBeInTheDocument();
  });

  it('has accessible aria-label', () => {
    render(<StatusBadge status="ready" />);
    expect(screen.getByLabelText('Status: Ready')).toBeInTheDocument();
  });

  it('applies correct aria-label for error status', () => {
    render(<StatusBadge status="error" />);
    expect(screen.getByLabelText('Status: Error')).toBeInTheDocument();
  });
});
