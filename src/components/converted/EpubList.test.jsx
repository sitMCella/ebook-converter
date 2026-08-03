import { describe, it, expect } from 'vitest';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { EpubList } from './EpubList';

const files = [
  {
    path: '/docs/design-patterns.pdf',
    name: 'Design patterns.pdf',
    outputPath: '/output/Design patterns.epub',
    conversionResult: { fileSize: 3250585 },
  },
  {
    path: '/docs/pragmatic.pdf',
    name: 'Pragmatic programmer.pdf',
    outputPath: '/output/Pragmatic programmer.epub',
    conversionResult: { fileSize: 4194304 },
  },
];

describe('EpubList', () => {
  it('renders a listbox with correct aria label', () => {
    render(<EpubList files={files} selectedPath={null} onSelect={() => {}} />);
    const listbox = screen.getByRole('listbox');
    expect(listbox).toHaveAttribute('aria-label', 'Converted EPUB list');
  });

  it('renders one option per file', () => {
    render(<EpubList files={files} selectedPath={null} onSelect={() => {}} />);
    const options = screen.getAllByRole('option');
    expect(options).toHaveLength(2);
  });

  it('marks the selected file', () => {
    render(
      <EpubList files={files} selectedPath="/docs/pragmatic.pdf" onSelect={() => {}} />,
    );
    const options = screen.getAllByRole('option');
    expect(options[0]).toHaveAttribute('aria-selected', 'false');
    expect(options[1]).toHaveAttribute('aria-selected', 'true');
  });

  it('calls onSelect with file path when an item is clicked', async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    render(<EpubList files={files} selectedPath={null} onSelect={onSelect} />);

    const listbox = screen.getByRole('listbox');
    await user.click(within(listbox).getByText('Pragmatic programmer.epub'));
    expect(onSelect).toHaveBeenCalledWith('/docs/pragmatic.pdf');
  });

  it('renders empty listbox when files array is empty', () => {
    render(<EpubList files={[]} selectedPath={null} onSelect={() => {}} />);
    const listbox = screen.getByRole('listbox');
    expect(within(listbox).queryAllByRole('option')).toHaveLength(0);
  });
});
