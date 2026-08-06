import { test, expect } from '@playwright/test';

function createFakeFile(name) {
  return {
    name,
    mimeType: 'application/pdf',
    buffer: Buffer.from('%PDF-1.4 fake content'),
  };
}

async function stageFile(page, fileName) {
  const fileChooserPromise = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: /browse files/i }).click();
  const fileChooser = await fileChooserPromise;
  await fileChooser.setFiles(createFakeFile(fileName));
  await expect(page.getByText(fileName)).toBeVisible();
}

async function stageFiles(page, fileNames) {
  const fileChooserPromise = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: /browse files/i }).click();
  const fileChooser = await fileChooserPromise;
  await fileChooser.setFiles(fileNames.map(createFakeFile));
  for (const name of fileNames) {
    await expect(page.getByText(name)).toBeVisible();
  }
}

async function importAllStagedToLibrary(page) {
  for (const checkbox of await page.getByRole('checkbox').all()) {
    await checkbox.check();
  }
  await page.getByRole('button', { name: /import to library/i }).click();
  await expect(page.getByText('No files staged yet.')).toBeVisible();
}

async function stageAndImportToLibrary(page, fileNames) {
  await page.goto('/import');
  if (Array.isArray(fileNames)) {
    await stageFiles(page, fileNames);
  } else {
    await stageFile(page, fileNames);
  }
  await importAllStagedToLibrary(page);
  await page.locator('nav').getByText('Library').click();
  await expect(page).toHaveURL(/\/library/);
}

// --- Empty State ---

test.describe('Library Screen — Empty State', () => {
  test('shows empty message when no files are imported', async ({ page }) => {
    await page.goto('/library');
    await expect(page.getByText('Your library is empty. Import some PDFs to get started.')).toBeVisible();
  });

  test('shows "Go to Import" button in empty state', async ({ page }) => {
    await page.goto('/library');
    await expect(page.getByRole('button', { name: /go to import/i })).toBeVisible();
  });

  test('"Go to Import" button navigates to import screen', async ({ page }) => {
    await page.goto('/library');
    await page.getByRole('button', { name: /go to import/i }).click();
    await expect(page).toHaveURL(/\/import/);
  });
});

// --- Layout with Files ---

test.describe('Library Screen — Layout', () => {
  test.beforeEach(async ({ page }) => {
    await stageAndImportToLibrary(page, ['alpha.pdf', 'beta.pdf']);
  });

  test('shows the Library heading', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Library' })).toBeVisible();
  });

  test('shows the search input', async ({ page }) => {
    await expect(page.getByRole('textbox', { name: /search documents/i })).toBeVisible();
  });

  test('shows a document list with imported files', async ({ page }) => {
    const listbox = page.getByRole('listbox', { name: /document list/i });
    await expect(listbox).toBeVisible();
    await expect(listbox.getByText('alpha.pdf')).toBeVisible();
    await expect(listbox.getByText('beta.pdf')).toBeVisible();
  });

  test('auto-selects the first file in the list', async ({ page }) => {
    const firstOption = page.getByRole('option').first();
    await expect(firstOption).toHaveAttribute('aria-selected', 'true');
  });

  test('sidebar highlights Library as active nav item', async ({ page }) => {
    const libraryLink = page.locator('nav a[href="/library"]');
    await expect(libraryLink).toHaveClass(/font-medium/);
  });
});

// --- Document Selection ---

test.describe('Library Screen — Document Selection', () => {
  test.beforeEach(async ({ page }) => {
    await stageAndImportToLibrary(page, ['doc-one.pdf', 'doc-two.pdf']);
  });

  test('clicking a document selects it and deselects others', async ({ page }) => {
    const options = page.getByRole('option');
    await expect(options.first()).toHaveAttribute('aria-selected', 'true');
    await expect(options.nth(1)).toHaveAttribute('aria-selected', 'false');

    await options.nth(1).click();
    await expect(options.first()).toHaveAttribute('aria-selected', 'false');
    await expect(options.nth(1)).toHaveAttribute('aria-selected', 'true');
  });

  test('selected document shows its details in the detail panel', async ({ page }) => {
    await page.getByRole('option').filter({ hasText: 'doc-two.pdf' }).click();
    await expect(page.getByText('Page preview not yet available')).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Metadata' })).toBeVisible();
  });
});

// --- Search ---

test.describe('Library Screen — Search', () => {
  test.beforeEach(async ({ page }) => {
    await stageAndImportToLibrary(page, ['report-2024.pdf', 'invoice-jan.pdf', 'report-2025.pdf']);
  });

  test('filters documents by search query', async ({ page }) => {
    await page.getByRole('textbox', { name: /search documents/i }).fill('report');

    const listbox = page.getByRole('listbox', { name: /document list/i });
    await expect(listbox.getByText('report-2024.pdf')).toBeVisible();
    await expect(listbox.getByText('report-2025.pdf')).toBeVisible();
    await expect(listbox.getByText('invoice-jan.pdf')).not.toBeVisible();
  });

  test('search is case-insensitive', async ({ page }) => {
    await page.getByRole('textbox', { name: /search documents/i }).fill('REPORT');

    const listbox = page.getByRole('listbox', { name: /document list/i });
    await expect(listbox.getByText('report-2024.pdf')).toBeVisible();
    await expect(listbox.getByText('report-2025.pdf')).toBeVisible();
  });

  test('shows "No documents match your search" when nothing matches', async ({ page }) => {
    await page.getByRole('textbox', { name: /search documents/i }).fill('nonexistent');
    await expect(page.getByText('No documents match your search.')).toBeVisible();
  });

  test('clearing search shows all documents again', async ({ page }) => {
    const searchInput = page.getByRole('textbox', { name: /search documents/i });
    await searchInput.fill('report');
    await expect(page.getByRole('listbox').getByText('invoice-jan.pdf')).not.toBeVisible();

    await searchInput.clear();
    await expect(page.getByRole('listbox').getByText('invoice-jan.pdf')).toBeVisible();
    await expect(page.getByRole('listbox').getByText('report-2024.pdf')).toBeVisible();
  });

  test('auto-selects first match when current selection is filtered out', async ({ page }) => {
    await page.getByRole('option').filter({ hasText: 'invoice-jan.pdf' }).click();
    await expect(page.getByRole('option').filter({ hasText: 'invoice-jan.pdf' })).toHaveAttribute('aria-selected', 'true');

    await page.getByRole('textbox', { name: /search documents/i }).fill('report');
    const firstOption = page.getByRole('option').first();
    await expect(firstOption).toHaveAttribute('aria-selected', 'true');
    await expect(firstOption).toContainText('report');
  });
});

// --- Detail Panel ---

test.describe('Library Screen — Detail Panel', () => {
  test.beforeEach(async ({ page }) => {
    await stageAndImportToLibrary(page, 'details-test.pdf');
  });

  test('shows page preview placeholder', async ({ page }) => {
    await expect(page.getByText('Page preview not yet available')).toBeVisible();
  });

  test('shows metadata section heading', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Metadata' })).toBeVisible();
  });

  test('shows format metadata from PDF validation', async ({ page }) => {
    await expect(page.getByText('Format')).toBeVisible();
    await expect(page.getByText(/PDF \d/)).toBeVisible();
  });

  test('shows "Convert to EPUB" button', async ({ page }) => {
    await expect(page.getByRole('button', { name: /convert to epub/i })).toBeVisible();
  });

  test('convert button navigates to converting screen', async ({ page }) => {
    await page.getByRole('button', { name: /convert to epub/i }).click();
    await expect(page).toHaveURL(/\/converting/);
  });
});

// --- Conversion Options ---

test.describe('Library Screen — Conversion Options', () => {
  test.beforeEach(async ({ page }) => {
    await stageAndImportToLibrary(page, 'options-test.pdf');
  });

  test('conversion options section is collapsed by default', async ({ page }) => {
    await expect(page.getByText('Conversion options')).toBeVisible();
    await expect(page.getByText('Heading level threshold')).not.toBeVisible();
  });

  test('clicking header expands conversion options', async ({ page }) => {
    await page.getByText('Conversion options').click();
    await expect(page.getByText('Heading level threshold')).toBeVisible();
    await expect(page.getByText('Base font size')).toBeVisible();
    await expect(page.getByText('Image quality')).toBeVisible();
    await expect(page.getByText('Page range')).toBeVisible();
  });

  test('clicking header again collapses conversion options', async ({ page }) => {
    await page.getByText('Conversion options').click();
    await expect(page.getByText('Heading level threshold')).toBeVisible();

    await page.getByText('Conversion options').click();
    await expect(page.getByText('Heading level threshold')).not.toBeVisible();
  });

  test('changing an option shows override count badge', async ({ page }) => {
    await page.getByText('Conversion options').click();

    const qualitySelect = page.locator('select').first();
    await qualitySelect.selectOption('low');

    await expect(page.getByText('1 custom')).toBeVisible();
  });

  test('reset button clears override and removes count badge', async ({ page }) => {
    await page.getByText('Conversion options').click();

    const qualitySelect = page.locator('select').first();
    await qualitySelect.selectOption('low');
    await expect(page.getByText('1 custom')).toBeVisible();

    await page.getByRole('button', { name: /reset to default/i }).click();
    await expect(page.getByText('1 custom')).not.toBeVisible();
  });
});

// --- State Persistence across Navigation ---

test.describe('Library Screen — State Persistence', () => {
  test('files persist when navigating away and back', async ({ page }) => {
    await stageAndImportToLibrary(page, ['persist-a.pdf', 'persist-b.pdf']);

    await page.locator('nav').getByText('Import').click();
    await expect(page).toHaveURL(/\/import/);

    await page.locator('nav').getByText('Library').click();
    await expect(page).toHaveURL(/\/library/);

    const listbox = page.getByRole('listbox', { name: /document list/i });
    await expect(listbox.getByText('persist-a.pdf')).toBeVisible();
    await expect(listbox.getByText('persist-b.pdf')).toBeVisible();
  });

  test('search query resets when navigating away and back', async ({ page }) => {
    await stageAndImportToLibrary(page, ['reset-a.pdf', 'reset-b.pdf']);

    await page.getByRole('textbox', { name: /search documents/i }).fill('reset-a');
    await expect(page.getByRole('listbox').getByText('reset-b.pdf')).not.toBeVisible();

    await page.locator('nav').getByText('Import').click();
    await page.locator('nav').getByText('Library').click();

    await expect(page.getByRole('listbox').getByText('reset-a.pdf')).toBeVisible();
    await expect(page.getByRole('listbox').getByText('reset-b.pdf')).toBeVisible();
    await expect(page.getByRole('textbox', { name: /search documents/i })).toHaveValue('');
  });
});
