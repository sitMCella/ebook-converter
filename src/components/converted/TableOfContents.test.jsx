import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TableOfContents } from './TableOfContents';

describe('TableOfContents', () => {
  it('renders the toggle button', () => {
    render(<TableOfContents entries={[]} />);
    expect(screen.getByText('Table of contents')).toBeInTheDocument();
  });

  it('starts collapsed', () => {
    render(<TableOfContents entries={[{ title: 'Chapter 1', level: 1 }]} />);
    expect(screen.queryByText('Chapter 1')).not.toBeInTheDocument();
  });

  it('shows empty state when expanded with no entries', async () => {
    const user = userEvent.setup();
    render(<TableOfContents entries={[]} />);

    await user.click(screen.getByText('Table of contents'));
    expect(screen.getByText('No chapters detected.')).toBeInTheDocument();
  });

  it('shows chapter entries when expanded', async () => {
    const user = userEvent.setup();
    const entries = [
      { title: 'Introduction', level: 1 },
      { title: 'Getting Started', level: 1 },
      { title: 'Conclusion', level: 1 },
    ];
    render(<TableOfContents entries={entries} />);

    await user.click(screen.getByText('Table of contents'));
    expect(screen.getByText('Introduction')).toBeInTheDocument();
    expect(screen.getByText('Getting Started')).toBeInTheDocument();
    expect(screen.getByText('Conclusion')).toBeInTheDocument();
  });

  it('shows entry count badge', () => {
    const entries = [
      { title: 'Chapter 1', level: 1 },
      { title: 'Chapter 2', level: 1 },
    ];
    render(<TableOfContents entries={entries} />);
    expect(screen.getByText('(2)')).toBeInTheDocument();
  });

  it('does not show count badge when empty', () => {
    render(<TableOfContents entries={[]} />);
    expect(screen.queryByText('(0)')).not.toBeInTheDocument();
  });

  it('collapses again on second click', async () => {
    const user = userEvent.setup();
    const entries = [{ title: 'Chapter 1', level: 1 }];
    render(<TableOfContents entries={entries} />);

    const button = screen.getByText('Table of contents');
    await user.click(button);
    expect(screen.getByText('Chapter 1')).toBeInTheDocument();

    await user.click(button);
    expect(screen.queryByText('Chapter 1')).not.toBeInTheDocument();
  });

  it('defaults entries to empty array', async () => {
    const user = userEvent.setup();
    render(<TableOfContents />);

    await user.click(screen.getByText('Table of contents'));
    expect(screen.getByText('No chapters detected.')).toBeInTheDocument();
  });
});
