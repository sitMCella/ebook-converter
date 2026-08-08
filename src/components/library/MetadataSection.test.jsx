import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MetadataSection } from './MetadataSection';

const baseFile = {
  path: '/docs/design-patterns.pdf',
  name: 'Design patterns.pdf',
  size: 13003776,
  status: 'ready',
  metadata: {
    title: 'Design Patterns',
    author: 'Gamma, Helm, Johnson, Vlissides',
    pageCount: 384,
    pdfVersion: '1.7',
    createdDate: '1994-10-21',
    modifiedDate: '2004-03-15',
    producer: 'Adobe Acrobat 6.0',
    fileSize: 13003776,
  },
};

describe('MetadataSection', () => {
  describe('collapsed summary', () => {
    it('shows title and page count when collapsed', () => {
      render(<MetadataSection file={baseFile} />);
      expect(screen.getByText(/Design Patterns · 384 pages/)).toBeInTheDocument();
    });

    it('does not include file size in collapsed summary', () => {
      render(<MetadataSection file={baseFile} />);
      const summary = screen.getByText(/Design Patterns · 384 pages/);
      expect(summary.textContent).not.toMatch(/MB/);
    });

    it('shows only title when page count is absent', () => {
      const file = { ...baseFile, metadata: { ...baseFile.metadata, pageCount: undefined } };
      render(<MetadataSection file={file} />);
      expect(screen.getByText(/· Design Patterns$/)).toBeInTheDocument();
    });

    it('shows only page count when title is absent', () => {
      const file = { ...baseFile, metadata: { ...baseFile.metadata, title: null } };
      render(<MetadataSection file={file} />);
      expect(screen.getByText(/· 384 pages$/)).toBeInTheDocument();
    });

    it('shows no summary when both title and page count are absent', () => {
      const file = { ...baseFile, metadata: { fileSize: 1024 } };
      render(<MetadataSection file={file} />);
      const button = screen.getByRole('button', { name: /metadata/i });
      expect(button.textContent.trim()).toBe('Metadata');
    });
  });

  describe('expanded rows', () => {
    it('shows file size when expanded', async () => {
      const user = userEvent.setup();
      render(<MetadataSection file={baseFile} />);

      await user.click(screen.getByRole('button', { name: /metadata/i }));

      expect(screen.getByText('File size')).toBeInTheDocument();
      expect(screen.getByText('12.4 MB')).toBeInTheDocument();
    });

    it('shows all metadata rows when expanded', async () => {
      const user = userEvent.setup();
      render(<MetadataSection file={baseFile} />);

      await user.click(screen.getByRole('button', { name: /metadata/i }));

      expect(screen.getByText('Title')).toBeInTheDocument();
      expect(screen.getByText('Authors')).toBeInTheDocument();
      expect(screen.getByText('Pages')).toBeInTheDocument();
      expect(screen.getByText('Format')).toBeInTheDocument();
      expect(screen.getByText('Created')).toBeInTheDocument();
      expect(screen.getByText('Modified')).toBeInTheDocument();
      expect(screen.getByText('Producer')).toBeInTheDocument();
    });

    it('hides summary when expanded', async () => {
      const user = userEvent.setup();
      render(<MetadataSection file={baseFile} />);

      await user.click(screen.getByRole('button', { name: /metadata/i }));

      expect(screen.queryByText(/Design Patterns · 384 pages/)).not.toBeInTheDocument();
    });

    it('shows "No metadata available" when metadata is empty', async () => {
      const user = userEvent.setup();
      const file = { ...baseFile, size: 0, metadata: {} };
      render(<MetadataSection file={file} />);

      await user.click(screen.getByRole('button', { name: /metadata/i }));

      expect(screen.getByText(/no metadata available/i)).toBeInTheDocument();
    });
  });
});
