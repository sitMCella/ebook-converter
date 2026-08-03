import { test, expect } from '@playwright/test';

function createFakeFile(name) {
  return {
    name,
    mimeType: 'application/pdf',
    buffer: Buffer.from('%PDF-1.4 fake content'),
  };
}

async function importFile(page, fileName) {
  const fileChooserPromise = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: /browse files/i }).click();
  const fileChooser = await fileChooserPromise;
  await fileChooser.setFiles(createFakeFile(fileName));
  await expect(page.getByText(fileName)).toBeVisible();
}

async function importFiles(page, fileNames) {
  const fileChooserPromise = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: /browse files/i }).click();
  const fileChooser = await fileChooserPromise;
  await fileChooser.setFiles(fileNames.map(createFakeFile));
  for (const name of fileNames) {
    await expect(page.getByText(name)).toBeVisible();
  }
}

async function selectAndConvert(page, fileNames) {
  for (const name of fileNames) {
    await page.getByRole('checkbox', { name: new RegExp(`select ${name}`, 'i') }).click();
  }
  await page.getByRole('button', { name: /convert selected/i }).click();
}

function mockSuccessfulConversion(page) {
  return page.route(/\/src\/lib\/tauri\.js/, async (route) => {
    const response = await route.fetch();
    const body = await response.text();
    const modified = body.replace(
      'return Promise.reject(new Error("Conversion requires the desktop app"))',
      `return Promise.resolve({
        outputPath: '/fake/output/' + path.replace(/\\.pdf$/i, '.epub'),
        fileSize: 1048576,
        chapters: 12,
        images: 5,
      })`,
    );
    await route.fulfill({ body: modified, headers: response.headers() });
  });
}

async function importConvertAndNavigate(page, fileNames) {
  await mockSuccessfulConversion(page);
  await page.goto('/import');
  const names = Array.isArray(fileNames) ? fileNames : [fileNames];
  if (names.length === 1) {
    await importFile(page, names[0]);
  } else {
    await importFiles(page, names);
  }
  await selectAndConvert(page, names);
  await expect(page).toHaveURL(/\/converting/);
  await expect(
    page.getByRole('heading', { name: 'Conversion complete' }),
  ).toBeVisible();
  await page.getByRole('button', { name: /view converted/i }).click();
  await expect(page).toHaveURL(/\/converted/);
}

// --- Empty State ---

test.describe('Converted Screen — Empty State', () => {
  test('shows empty message when no files are converted', async ({ page }) => {
    await page.goto('/converted');
    await expect(
      page.getByText('No converted files yet. Import and convert a PDF to see it here.'),
    ).toBeVisible();
  });

  test('shows "Go to Import" button in empty state', async ({ page }) => {
    await page.goto('/converted');
    await expect(page.getByRole('button', { name: /go to import/i })).toBeVisible();
  });

  test('"Go to Import" button navigates to import screen', async ({ page }) => {
    await page.goto('/converted');
    await page.getByRole('button', { name: /go to import/i }).click();
    await expect(page).toHaveURL(/\/import/);
  });
});

// --- Layout with Converted Files ---

test.describe('Converted Screen — Layout', () => {
  test.beforeEach(async ({ page }) => {
    await importConvertAndNavigate(page, ['alpha.pdf', 'beta.pdf']);
  });

  test('shows the Converted EPUBs heading', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Converted EPUBs' })).toBeVisible();
  });

  test('shows the search input', async ({ page }) => {
    await expect(page.getByRole('textbox', { name: /search converted/i })).toBeVisible();
  });

  test('shows EPUB list with converted files using .epub extension', async ({ page }) => {
    const listbox = page.getByRole('listbox', { name: /converted epub list/i });
    await expect(listbox).toBeVisible();
    await expect(listbox.getByText('alpha.epub')).toBeVisible();
    await expect(listbox.getByText('beta.epub')).toBeVisible();
  });

  test('auto-selects the first file in the list', async ({ page }) => {
    const firstOption = page.getByRole('option').first();
    await expect(firstOption).toHaveAttribute('aria-selected', 'true');
  });

  test('sidebar highlights Converted as active nav item', async ({ page }) => {
    const convertedLink = page.locator('nav a[href="/converted"]');
    await expect(convertedLink).toHaveClass(/font-medium/);
  });
});

// --- EPUB Selection ---

test.describe('Converted Screen — EPUB Selection', () => {
  test.beforeEach(async ({ page }) => {
    await importConvertAndNavigate(page, ['doc-one.pdf', 'doc-two.pdf']);
  });

  test('clicking an EPUB selects it and deselects others', async ({ page }) => {
    const options = page.getByRole('option');
    await expect(options.first()).toHaveAttribute('aria-selected', 'true');
    await expect(options.nth(1)).toHaveAttribute('aria-selected', 'false');

    await options.nth(1).click();
    await expect(options.first()).toHaveAttribute('aria-selected', 'false');
    await expect(options.nth(1)).toHaveAttribute('aria-selected', 'true');
  });

  test('selected EPUB shows its details in the detail panel', async ({ page }) => {
    await page.getByRole('option').filter({ hasText: 'doc-two.epub' }).click();
    await expect(page.getByText('EPUB preview not yet available')).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Metadata' })).toBeVisible();
  });
});

// --- Search ---

test.describe('Converted Screen — Search', () => {
  test.beforeEach(async ({ page }) => {
    await importConvertAndNavigate(page, ['report-2024.pdf', 'invoice-jan.pdf', 'report-2025.pdf']);
  });

  test('filters EPUBs by search query', async ({ page }) => {
    await page.getByRole('textbox', { name: /search converted/i }).fill('report');

    const listbox = page.getByRole('listbox', { name: /converted epub list/i });
    await expect(listbox.getByText('report-2024.epub')).toBeVisible();
    await expect(listbox.getByText('report-2025.epub')).toBeVisible();
    await expect(listbox.getByText('invoice-jan.epub')).not.toBeVisible();
  });

  test('search is case-insensitive', async ({ page }) => {
    await page.getByRole('textbox', { name: /search converted/i }).fill('REPORT');

    const listbox = page.getByRole('listbox', { name: /converted epub list/i });
    await expect(listbox.getByText('report-2024.epub')).toBeVisible();
    await expect(listbox.getByText('report-2025.epub')).toBeVisible();
  });

  test('shows "No converted files match" message when nothing matches', async ({ page }) => {
    await page.getByRole('textbox', { name: /search converted/i }).fill('nonexistent');
    await expect(page.getByText('No converted files match your search.')).toBeVisible();
  });

  test('clearing search shows all EPUBs again', async ({ page }) => {
    const searchInput = page.getByRole('textbox', { name: /search converted/i });
    await searchInput.fill('report');
    await expect(page.getByRole('listbox').getByText('invoice-jan.epub')).not.toBeVisible();

    await searchInput.clear();
    await expect(page.getByRole('listbox').getByText('invoice-jan.epub')).toBeVisible();
    await expect(page.getByRole('listbox').getByText('report-2024.epub')).toBeVisible();
  });

  test('auto-selects first match when current selection is filtered out', async ({ page }) => {
    await page.getByRole('option').filter({ hasText: 'invoice-jan.epub' }).click();
    await expect(
      page.getByRole('option').filter({ hasText: 'invoice-jan.epub' }),
    ).toHaveAttribute('aria-selected', 'true');

    await page.getByRole('textbox', { name: /search converted/i }).fill('report');
    const firstOption = page.getByRole('option').first();
    await expect(firstOption).toHaveAttribute('aria-selected', 'true');
    await expect(firstOption).toContainText('report');
  });
});

// --- Detail Panel — Preview ---

test.describe('Converted Screen — Preview', () => {
  test.beforeEach(async ({ page }) => {
    await importConvertAndNavigate(page, 'preview-test.pdf');
  });

  test('shows EPUB preview placeholder', async ({ page }) => {
    await expect(page.getByText('EPUB preview not yet available')).toBeVisible();
  });

  test('shows chapter count in preview area', async ({ page }) => {
    await expect(page.getByText('12 chapters')).toBeVisible();
  });
});

// --- Detail Panel — Metadata ---

test.describe('Converted Screen — Metadata', () => {
  test.beforeEach(async ({ page }) => {
    await importConvertAndNavigate(page, 'metadata-test.pdf');
  });

  test('shows metadata section heading', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Metadata' })).toBeVisible();
  });

  test('shows source PDF file name', async ({ page }) => {
    await expect(page.getByText('Source')).toBeVisible();
    await expect(page.getByText('metadata-test.pdf')).toBeVisible();
  });

  test('shows EPUB file size', async ({ page }) => {
    await expect(page.getByText('EPUB size')).toBeVisible();
    const metadataSection = page.getByRole('heading', { name: 'Metadata' }).locator('..');
    await expect(metadataSection.getByText('1.0 MB')).toBeVisible();
  });

  test('shows chapter count in metadata', async ({ page }) => {
    await expect(page.getByText('Chapters', { exact: true })).toBeVisible();
  });

  test('shows extracted image count', async ({ page }) => {
    await expect(page.getByText('Images')).toBeVisible();
    await expect(page.getByText('5 extracted')).toBeVisible();
  });

  test('shows settings used label', async ({ page }) => {
    await expect(page.getByText('Settings used')).toBeVisible();
    await expect(page.getByText('Default')).toBeVisible();
  });
});

// --- Detail Panel — Table of Contents ---

test.describe('Converted Screen — Table of Contents', () => {
  test.beforeEach(async ({ page }) => {
    await importConvertAndNavigate(page, 'toc-test.pdf');
  });

  test('shows Table of contents section collapsed', async ({ page }) => {
    await expect(page.getByRole('button', { name: 'Table of contents' })).toBeVisible();
    await expect(page.getByText('Table of contents not yet available.')).not.toBeVisible();
  });

  test('clicking expands Table of contents with placeholder', async ({ page }) => {
    await page.getByRole('button', { name: 'Table of contents' }).click();
    await expect(page.getByText('Table of contents not yet available.')).toBeVisible();
  });

  test('clicking again collapses Table of contents', async ({ page }) => {
    await page.getByRole('button', { name: 'Table of contents' }).click();
    await expect(page.getByText('Table of contents not yet available.')).toBeVisible();

    await page.getByRole('button', { name: 'Table of contents' }).click();
    await expect(page.getByText('Table of contents not yet available.')).not.toBeVisible();
  });
});

// --- Detail Panel — Action Buttons ---

test.describe('Converted Screen — Action Buttons', () => {
  test.beforeEach(async ({ page }) => {
    await importConvertAndNavigate(page, 'actions-test.pdf');
  });

  test('shows Reconvert button (browser mode)', async ({ page }) => {
    await expect(page.getByRole('button', { name: /reconvert/i })).toBeVisible();
  });

  test('hides Open in reader button in browser mode', async ({ page }) => {
    await expect(page.getByRole('button', { name: /open in reader/i })).not.toBeVisible();
  });

  test('hides Save as button in browser mode', async ({ page }) => {
    await expect(page.getByRole('button', { name: /save as/i })).not.toBeVisible();
  });

  test('hides Open folder button in browser mode', async ({ page }) => {
    await expect(page.getByRole('button', { name: /open folder/i })).not.toBeVisible();
  });

  test('Reconvert button navigates to Library with source PDF selected', async ({ page }) => {
    await page.getByRole('button', { name: /reconvert/i }).click();
    await expect(page).toHaveURL(/\/library/);
  });
});

// --- Navigation from Converting Screen ---

test.describe('Converted Screen — Navigation from Converting', () => {
  test('View converted button navigates to converted screen', async ({ page }) => {
    await mockSuccessfulConversion(page);
    await page.goto('/import');
    await importFile(page, 'nav-test.pdf');
    await selectAndConvert(page, ['nav-test.pdf']);

    await expect(
      page.getByRole('heading', { name: 'Conversion complete' }),
    ).toBeVisible();
    await page.getByRole('button', { name: /view converted/i }).click();
    await expect(page).toHaveURL(/\/converted/);
    await expect(page.getByText('nav-test.epub')).toBeVisible();
  });

  test('clicking completed row navigates to converted screen with file selected', async ({ page }) => {
    await mockSuccessfulConversion(page);
    await page.goto('/import');
    await importFiles(page, ['first.pdf', 'second.pdf']);
    await selectAndConvert(page, ['first.pdf', 'second.pdf']);

    await expect(
      page.getByRole('heading', { name: 'Conversion complete' }),
    ).toBeVisible();

    await page
      .locator('.cursor-pointer')
      .filter({ hasText: 'second.pdf' })
      .click();
    await expect(page).toHaveURL(/\/converted/);

    const selectedOption = page.getByRole('option').filter({ hasText: 'second.epub' });
    await expect(selectedOption).toHaveAttribute('aria-selected', 'true');
  });
});

// --- Navigation via Sidebar ---

test.describe('Converted Screen — Sidebar Navigation', () => {
  test('Converted link in sidebar navigates to /converted', async ({ page }) => {
    await page.goto('/import');
    await page.locator('nav').getByText('Converted').click();
    await expect(page).toHaveURL(/\/converted/);
  });
});

// --- State Persistence across Navigation ---

test.describe('Converted Screen — State Persistence', () => {
  test('converted files persist when navigating away and back', async ({ page }) => {
    await importConvertAndNavigate(page, ['persist-a.pdf', 'persist-b.pdf']);

    await page.locator('nav').getByText('Import').click();
    await expect(page).toHaveURL(/\/import/);

    await page.locator('nav').getByText('Converted').click();
    await expect(page).toHaveURL(/\/converted/);

    const listbox = page.getByRole('listbox', { name: /converted epub list/i });
    await expect(listbox.getByText('persist-a.epub')).toBeVisible();
    await expect(listbox.getByText('persist-b.epub')).toBeVisible();
  });

  test('search query resets when navigating away and back', async ({ page }) => {
    await importConvertAndNavigate(page, ['reset-a.pdf', 'reset-b.pdf']);

    await page.getByRole('textbox', { name: /search converted/i }).fill('reset-a');
    await expect(page.getByRole('listbox').getByText('reset-b.epub')).not.toBeVisible();

    await page.locator('nav').getByText('Import').click();
    await page.locator('nav').getByText('Converted').click();

    await expect(page.getByRole('listbox').getByText('reset-a.epub')).toBeVisible();
    await expect(page.getByRole('listbox').getByText('reset-b.epub')).toBeVisible();
    await expect(page.getByRole('textbox', { name: /search converted/i })).toHaveValue('');
  });
});

// --- EPUB List Item Details ---

test.describe('Converted Screen — List Item Details', () => {
  test('shows EPUB file size in the list item', async ({ page }) => {
    await importConvertAndNavigate(page, 'sized-file.pdf');

    const listItem = page.getByRole('option').filter({ hasText: 'sized-file.epub' });
    await expect(listItem.getByText('1.0 MB')).toBeVisible();
  });
});
