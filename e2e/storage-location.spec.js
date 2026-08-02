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
      'async function deleteBook(bookId) {',
      [
        'async function deleteBook(bookId) {',
        'window.__deleteBookCalls = window.__deleteBookCalls || [];',
        'window.__deleteBookCalls.push(bookId);',
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

// --- Storage on Import ---

test.describe('Storage location — PDF import creates book in managed storage', () => {
  test.beforeEach(async ({ page }) => {
    await mockStorageOperations(page);
    await page.goto('/import');
  });

  test('importPdf is called when importing a file', async ({ page }) => {
    await importFile(page, 'storage-test.pdf');

    const calls = await page.evaluate(() => window.__importPdfCalls);
    expect(calls).toHaveLength(1);
    expect(calls[0].sourcePath).toBe('storage-test.pdf');
    expect(calls[0].bookId).toBe('test-uuid-0');
  });

  test('each imported file gets a unique bookId', async ({ page }) => {
    await importFiles(page, ['first.pdf', 'second.pdf', 'third.pdf']);

    const calls = await page.evaluate(() => window.__importPdfCalls);
    expect(calls).toHaveLength(3);

    const bookIds = calls.map((c) => c.bookId);
    const uniqueIds = new Set(bookIds);
    expect(uniqueIds.size).toBe(3);
  });

  test('file shows Ready status after storage import', async ({ page }) => {
    await importFile(page, 'ready-check.pdf');
    await expect(page.locator('[aria-label="Status: Ready"]')).toBeVisible();
  });
});

// --- Storage Cleanup on Remove ---

test.describe('Storage location — book deletion on file removal', () => {
  test.beforeEach(async ({ page }) => {
    await mockStorageOperations(page);
    await page.goto('/import');
  });

  test('deleteBook is called with correct bookId when removing a stored file', async ({ page }) => {
    await importFile(page, 'delete-test.pdf');

    const importCalls = await page.evaluate(() => window.__importPdfCalls);
    const bookId = importCalls[0].bookId;

    await page.getByRole('checkbox', { name: /select delete-test\.pdf/i }).check();
    await page.getByRole('button', { name: /remove selected/i }).click();
    await page.getByRole('dialog').getByRole('button', { name: /confirm/i }).click();

    await expect(page.getByText('delete-test.pdf')).not.toBeVisible();

    const deleteCalls = await page.evaluate(() => window.__deleteBookCalls);
    expect(deleteCalls).toHaveLength(1);
    expect(deleteCalls[0]).toBe(bookId);
  });

  test('deleteBook is called for each file in batch removal', async ({ page }) => {
    await importFiles(page, ['remove-a.pdf', 'remove-b.pdf']);

    await page.getByRole('checkbox', { name: /select remove-a\.pdf/i }).check();
    await page.getByRole('checkbox', { name: /select remove-b\.pdf/i }).check();
    await page.getByRole('button', { name: /remove selected/i }).click();
    await page.getByRole('dialog').getByRole('button', { name: /confirm/i }).click();

    await expect(page.getByText('remove-a.pdf')).not.toBeVisible();
    await expect(page.getByText('remove-b.pdf')).not.toBeVisible();

    const deleteCalls = await page.evaluate(() => window.__deleteBookCalls);
    expect(deleteCalls).toHaveLength(2);
  });

  test('removal succeeds even if deleteBook fails gracefully', async ({ page }) => {
    await importFile(page, 'fail-delete.pdf');

    await page.getByRole('checkbox', { name: /select fail-delete\.pdf/i }).check();
    await page.getByRole('button', { name: /remove selected/i }).click();
    await page.getByRole('dialog').getByRole('button', { name: /confirm/i }).click();

    await expect(page.getByText('fail-delete.pdf')).not.toBeVisible();
    await expect(page.getByText('No files imported yet.')).toBeVisible();
  });
});

// --- Conversion with Book Storage ---

test.describe('Storage location — conversion uses book directory', () => {
  test.beforeEach(async ({ page }) => {
    await mockStorageOperations(page);
    await page.goto('/import');
  });

  test('bookId is included in conversion options for stored files', async ({ page }) => {
    await importFile(page, 'convert-stored.pdf');
    await selectAndConvert(page, ['convert-stored.pdf']);

    await expect(page.getByRole('heading', { name: 'Conversion complete' })).toBeVisible();

    const convertCalls = await page.evaluate(() => window.__convertCalls);
    expect(convertCalls).toHaveLength(1);
    expect(convertCalls[0].options.bookId).toBe('test-uuid-0');
  });

  test('storedPdfPath is used as conversion input path', async ({ page }) => {
    await importFile(page, 'stored-path.pdf');
    await selectAndConvert(page, ['stored-path.pdf']);

    await expect(page.getByRole('heading', { name: 'Conversion complete' })).toBeVisible();

    const convertCalls = await page.evaluate(() => window.__convertCalls);
    expect(convertCalls).toHaveLength(1);
    expect(convertCalls[0].path).toBe('/app-data/books/test-uuid-0/source.pdf');
  });

  test('batch conversion passes correct bookId for each file', async ({ page }) => {
    await importFiles(page, ['batch-a.pdf', 'batch-b.pdf']);
    await selectAndConvert(page, ['batch-a.pdf', 'batch-b.pdf']);

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
