import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Toggle } from './Toggle';

describe('Toggle', () => {
  it('renders unchecked state with correct aria attributes', () => {
    render(<Toggle checked={false} onChange={() => {}} label="Test toggle" />);
    const toggle = screen.getByRole('switch');
    expect(toggle).toHaveAttribute('aria-checked', 'false');
    expect(toggle).toHaveAttribute('aria-label', 'Test toggle');
  });

  it('renders checked state', () => {
    render(<Toggle checked={true} onChange={() => {}} label="Test toggle" />);
    expect(screen.getByRole('switch')).toHaveAttribute('aria-checked', 'true');
  });

  it('clicking toggles the value', async () => {
    const onChange = vi.fn();
    render(<Toggle checked={false} onChange={onChange} label="Test toggle" />);
    await userEvent.click(screen.getByRole('switch'));
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it('disabled state prevents interaction', async () => {
    const onChange = vi.fn();
    render(<Toggle checked={false} onChange={onChange} label="Test toggle" disabled />);
    const toggle = screen.getByRole('switch');
    expect(toggle).toBeDisabled();
    await userEvent.click(toggle);
    expect(onChange).not.toHaveBeenCalled();
  });

  it('has accessible label', () => {
    render(<Toggle checked={false} onChange={() => {}} label="My setting" />);
    expect(screen.getByLabelText('My setting')).toBeInTheDocument();
  });
});
