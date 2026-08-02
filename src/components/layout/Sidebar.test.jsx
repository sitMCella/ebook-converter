import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { Sidebar } from './Sidebar';

function renderSidebar(initialRoute = '/import') {
  return render(
    <MemoryRouter initialEntries={[initialRoute]}>
      <Sidebar />
    </MemoryRouter>
  );
}

describe('Sidebar', () => {
  it('renders all navigation items', () => {
    renderSidebar();
    expect(screen.getByText('Import')).toBeInTheDocument();
    expect(screen.getByText('Library')).toBeInTheDocument();
    expect(screen.getByText('Converted')).toBeInTheDocument();
    expect(screen.getByText('Settings')).toBeInTheDocument();
  });

  it('renders the "Tools" section divider', () => {
    renderSidebar();
    expect(screen.getByText('Tools')).toBeInTheDocument();
  });

  it('highlights Import as active on /import', () => {
    renderSidebar('/import');
    const importLink = screen.getByText('Import').closest('a');
    expect(importLink.className).toContain('font-medium');
  });

  it('has correct link targets', () => {
    renderSidebar();
    expect(screen.getByText('Import').closest('a')).toHaveAttribute('href', '/import');
    expect(screen.getByText('Library').closest('a')).toHaveAttribute('href', '/library');
    expect(screen.getByText('Converted').closest('a')).toHaveAttribute('href', '/converted');
    expect(screen.getByText('Settings').closest('a')).toHaveAttribute('href', '/settings');
  });
});
