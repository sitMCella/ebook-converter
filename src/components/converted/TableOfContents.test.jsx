import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TableOfContents } from './TableOfContents';

describe('TableOfContents', () => {
  it('renders the toggle button', () => {
    render(<TableOfContents />);
    expect(screen.getByText('Table of contents')).toBeInTheDocument();
  });

  it('starts collapsed with no placeholder visible', () => {
    render(<TableOfContents />);
    expect(screen.queryByText(/not yet available/i)).not.toBeInTheDocument();
  });

  it('expands to show placeholder on click', async () => {
    const user = userEvent.setup();
    render(<TableOfContents />);

    await user.click(screen.getByText('Table of contents'));
    expect(screen.getByText('Table of contents not yet available.')).toBeInTheDocument();
  });

  it('collapses again on second click', async () => {
    const user = userEvent.setup();
    render(<TableOfContents />);

    const button = screen.getByText('Table of contents');
    await user.click(button);
    expect(screen.getByText('Table of contents not yet available.')).toBeInTheDocument();

    await user.click(button);
    expect(screen.queryByText('Table of contents not yet available.')).not.toBeInTheDocument();
  });
});
