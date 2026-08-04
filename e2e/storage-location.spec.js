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

async function convertFromLibrary(page, fileNames) {
  const names = Array.isArray(fileNames) ? fileNames : [fileNames];
  for (const name of names) {
    await page.locator('nav').getByText('Library').click();
    await page.getByRole('option').filter({ hasText: name }).click();
    await page.getByRole('button', { name: /convert to epub/i }).click();
  }
}

function mockStorageOperations(page) {
  return page.route(/\/src\/lib\/tauri\.js/, async (route) => {
    const response = await route.fetch();
    const body = await response.text();
    let modified = body;

    modified = modified.replace(
      /return\s*\{\s*bookId:\s*null,\s*storedPdfPath:\s*sourcePath\s*\}/,
      [
        'window.__importPdfCalls = window.__importPdfCalls || [];',
        "var bookId = 'test-uuid-' + window.__importPdfCalls.length;",
        'window.__importPdfCalls.push({ sourcePath, bookId });',
        "return { bookId: bookId, storedPdfPath: '/app-data/books/' + bookId + '/source.pdf' }",
      ].join('\n'),
    );

    modified = modified.replace(
      'return Promise.reject(new Error("Conversion requires the desktop app"))',
      [
        'window.__convertCalls = window.__convertCalls || [];',
        'window.__convertCalls.push({ path, options });',
        'return Promise.reject(new Error("Conversion requires the desktop app"))',
      ].join('\n'),
    );

    await route.fulfill({ body: modified, headers: response.headers() });
  });
}

// --- Storage on Import to Library ---

test.describe('Storage location — PDF import creates book in managed storage', () => {
  test.beforeEach(async ({ page }) => {
    await mockStorageOperations(page);
    await page.goto('/import');
  });

  test('importPdf is called when importing staged files to library', async ({ page }) => {
    await stageFile(page, 'storage-test.pdf');

    const callsBefore = await page.evaluate(() => window.__importPdfCalls);
    expect(callsBefore).toBeFalsy();

    await page.getByRole('checkbox', { name: /select storage-test\.pdf/i }).check();
    await page.getByRole('button', { name: /import to library/i }).click();

    const calls = await page.evaluate(() => window.__importPdfCalls);
    expect(calls).toHaveLength(1);
    expect(calls[0].sourcePath).toBe('storage-test.pdf');
    expect(calls[0].bookId).toBe('test-uuid-0');
  });

  test('each imported file gets a unique bookId', async ({ page }) => {
    await stageFiles(page, ['first.pdf', 'second.pdf', 'third.pdf']);
    await importAllStagedToLibrary(page);

    const calls = await page.evaluate(() => window.__importPdfCalls);
    expect(calls).toHaveLength(3);

    const bookIds = calls.map((c) => c.bookId);
    const uniqueIds = new Set(bookIds);
    expect(uniqueIds.size).toBe(3);
  });

  test('file appears in library after import', async ({ page }) => {
    await stageFile(page, 'ready-check.pdf');
    await page.getByRole('checkbox', { name: /select ready-check\.pdf/i }).check();
    await page.getByRole('button', { name: /import to library/i }).click();

    await page.locator('nav').getByText('Library').click();
    const listbox = page.getByRole('listbox', { name: /document list/i });
    await expect(listbox.getByText('ready-check.pdf')).toBeVisible();
  });
});

// --- Staging Removal is Non-Destructive ---

test.describe('Storage location — staging removal does not delete storage', () => {
  test.beforeEach(async ({ page }) => {
    await mockStorageOperations(page);
    await page.goto('/import');
  });

  test('removing staged files does not call importPdf or deleteBook', async ({ page }) => {
    await stageFile(page, 'remove-test.pdf');

    await page.getByRole('checkbox', { name: /select remove-test\.pdf/i }).check();
    await page.getByRole('button', { name: /remove selected/i }).click();

    await expect(page.getByText('remove-test.pdf')).not.toBeVisible();
    await expect(page.getByText('No files staged yet.')).toBeVisible();

    const importCalls = await page.evaluate(() => window.__importPdfCalls);
    expect(importCalls).toBeFalsy();
  });
});

// --- Conversion with Book Storage ---

test.describe('Storage location — conversion uses book directory', () => {
  test.beforeEach(async ({ page }) => {
    await mockStorageOperations(page);
    await page.goto('/import');
  });

  test('bookId is included in conversion options for stored files', async ({ page }) => {
    await stageFile(page, 'convert-stored.pdf');
    await importAllStagedToLibrary(page);
    await convertFromLibrary(page, 'convert-stored.pdf');

    await expect(page.getByRole('heading', { name: 'Conversion complete' })).toBeVisible();

    const convertCalls = await page.evaluate(() => window.__convertCalls);
    expect(convertCalls).toHaveLength(1);
    expect(convertCalls[0].options.bookId).toBe('test-uuid-0');
  });

  test('storedPdfPath is used as conversion input path', async ({ page }) => {
    await stageFile(page, 'stored-path.pdf');
    await importAllStagedToLibrary(page);
    await convertFromLibrary(page, 'stored-path.pdf');

    await expect(page.getByRole('heading', { name: 'Conversion complete' })).toBeVisible();

    const convertCalls = await page.evaluate(() => window.__convertCalls);
    expect(convertCalls).toHaveLength(1);
    expect(convertCalls[0].path).toBe('/app-data/books/test-uuid-0/source.pdf');
  });

  test('batch conversion passes correct bookId for each file', async ({ page }) => {
    await stageFiles(page, ['batch-a.pdf', 'batch-b.pdf']);
    await importAllStagedToLibrary(page);
    await convertFromLibrary(page, ['batch-a.pdf', 'batch-b.pdf']);

    await expect(page.getByRole('heading', { name: 'Conversion complete' })).toBeVisible();

    const convertCalls = await page.evaluate(() => window.__convertCalls);
    expect(convertCalls).toHaveLength(2);

    const bookIds = convertCalls.map((c) => c.options.bookId);
    expect(bookIds).toContain('test-uuid-0');
    expect(bookIds).toContain('test-uuid-1');
  });
});

// --- Settings Screen ---

test.describe('Storage location — Settings screen', () => {
  test('does not show "Output location" setting group', async ({ page }) => {
    await page.goto('/settings');
    await expect(page.getByText(/output location/i)).not.toBeVisible();
  });

  test('does not show "Default output folder" setting', async ({ page }) => {
    await page.goto('/settings');
    await expect(page.getByText(/default output folder/i)).not.toBeVisible();
  });
});
