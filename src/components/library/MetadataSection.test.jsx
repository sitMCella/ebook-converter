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
  describe('expanded rows', () => {
    it('shows file size when expanded', async () => {
      const user = userEvent.setup();
      render(<MetadataSection file={baseFile} />);

      await user.click(screen.getByRole('button', { name: /metadata/i }));

      expect(screen.getByText('File size')).toBeInTheDocument();
      expect(screen.getByText('12.4 MB')).toBeInTheDocument();
    });

    it('shows PDF file name when expanded', async () => {
      const user = userEvent.setup();
      render(<MetadataSection file={baseFile} />);

      await user.click(screen.getByRole('button', { name: /metadata/i }));

      expect(screen.getByText('File name')).toBeInTheDocument();
      expect(screen.getByText('Design patterns.pdf')).toBeInTheDocument();
    });

    it('shows all metadata rows when expanded', async () => {
      const user = userEvent.setup();
      render(<MetadataSection file={baseFile} />);

      await user.click(screen.getByRole('button', { name: /metadata/i }));

      expect(screen.getByText('File name')).toBeInTheDocument();
      expect(screen.getByText('Title')).toBeInTheDocument();
      expect(screen.getByText('Authors')).toBeInTheDocument();
      expect(screen.getByText('Pages')).toBeInTheDocument();
      expect(screen.getByText('Format')).toBeInTheDocument();
      expect(screen.getByText('Created')).toBeInTheDocument();
      expect(screen.getByText('Modified')).toBeInTheDocument();
      expect(screen.getByText('Producer')).toBeInTheDocument();
    });

    it('shows "No metadata available" when metadata is empty', async () => {
      const user = userEvent.setup();
      const file = { ...baseFile, name: '', size: 0, metadata: {} };
      render(<MetadataSection file={file} />);

      await user.click(screen.getByRole('button', { name: /metadata/i }));

      expect(screen.getByText(/no metadata available/i)).toBeInTheDocument();
    });
  });
});
