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

function mockHangingConversion(page) {
  return page.route(/\/src\/lib\/tauri\.js/, async (route) => {
    const response = await route.fetch();
    const body = await response.text();
    const modified = body.replace(
      'return Promise.reject(new Error("Conversion requires the desktop app"))',
      'return new Promise(() => {})',
    );
    await route.fulfill({ body: modified, headers: response.headers() });
  });
}

// --- Converting screen empty state ---

test.describe('Converting screen — empty state', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/converting');
  });

  test('shows Converting heading when no work is active', async ({ page }) => {
    await expect(
      page.getByRole('heading', { name: 'Converting' }),
    ).toBeVisible();
  });

  test('shows log waiting message', async ({ page }) => {
    await expect(page.getByText('Conversion log')).toBeVisible();
    await expect(
      page.getByText('Waiting for conversion to start...'),
    ).toBeVisible();
  });

  test('does not show Cancel all button', async ({ page }) => {
    await expect(
      page.getByRole('button', { name: /cancel all/i }),
    ).not.toBeVisible();
  });

  test('does not show View converted button', async ({ page }) => {
    await expect(
      page.getByRole('button', { name: /view converted/i }),
    ).not.toBeVisible();
  });

  test('log region has aria-live polite', async ({ page }) => {
    const logRegion = page.locator('[aria-label="Conversion log entries"]');
    await expect(logRegion).toHaveAttribute('aria-live', 'polite');
  });
});

// --- Single file conversion (error flow in browser mode) ---

test.describe('Single file conversion — error handling', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/import');
  });

  test('navigates to /converting after clicking Convert selected', async ({
    page,
  }) => {
    await importFile(page, 'document.pdf');
    await selectAndConvert(page, ['document.pdf']);
    await expect(page).toHaveURL(/\/converting/);
  });

  test('shows error badge when conversion fails', async ({ page }) => {
    await importFile(page, 'document.pdf');
    await selectAndConvert(page, ['document.pdf']);

    await expect(page.getByText('Completed')).toBeVisible();
    await expect(
      page.locator('[aria-label="Status: Error"]'),
    ).toBeVisible();
  });

  test('shows error message in conversion log', async ({ page }) => {
    await importFile(page, 'document.pdf');
    await selectAndConvert(page, ['document.pdf']);

    await expect(
      page.getByText(/Error:.*Conversion requires the desktop app/),
    ).toBeVisible();
  });

  test('shows Conversion complete heading after error', async ({ page }) => {
    await importFile(page, 'document.pdf');
    await selectAndConvert(page, ['document.pdf']);

    await expect(
      page.getByRole('heading', { name: 'Conversion complete' }),
    ).toBeVisible();
  });

  test('shows View converted button after completion', async ({ page }) => {
    await importFile(page, 'document.pdf');
    await selectAndConvert(page, ['document.pdf']);

    await expect(
      page.getByRole('button', { name: /view converted/i }),
    ).toBeVisible();
  });

  test('hides Cancel all button after conversion finishes', async ({
    page,
  }) => {
    await importFile(page, 'document.pdf');
    await selectAndConvert(page, ['document.pdf']);

    await expect(
      page.getByRole('heading', { name: 'Conversion complete' }),
    ).toBeVisible();
    await expect(
      page.getByRole('button', { name: /cancel all/i }),
    ).not.toBeVisible();
  });

  test('waiting message is replaced by log entries', async ({ page }) => {
    await importFile(page, 'document.pdf');
    await selectAndConvert(page, ['document.pdf']);

    await expect(
      page.getByText(/Error:.*Conversion requires the desktop app/),
    ).toBeVisible();
    await expect(
      page.getByText('Waiting for conversion to start...'),
    ).not.toBeVisible();
  });
});

// --- Batch conversion (error flow in browser mode) ---

test.describe('Batch conversion — error handling', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/import');
  });

  test('all files appear in completed list with error badges', async ({
    page,
  }) => {
    await importFiles(page, ['report.pdf', 'thesis.pdf', 'manual.pdf']);
    await selectAndConvert(page, ['report.pdf', 'thesis.pdf', 'manual.pdf']);

    await expect(page).toHaveURL(/\/converting/);
    await expect(
      page.getByRole('heading', { name: 'Conversion complete' }),
    ).toBeVisible();
    await expect(page.getByText('Completed')).toBeVisible();

    const errorBadges = page.locator('[aria-label="Status: Error"]');
    await expect(errorBadges).toHaveCount(3);
  });

  test('shows an error log entry for each failed file', async ({ page }) => {
    await importFiles(page, ['file1.pdf', 'file2.pdf', 'file3.pdf']);
    await selectAndConvert(page, ['file1.pdf', 'file2.pdf', 'file3.pdf']);

    await expect(
      page.getByRole('heading', { name: 'Conversion complete' }),
    ).toBeVisible();

    const errorEntries = page
      .locator('[aria-live="polite"] p')
      .filter({ hasText: /Error:/ });
    await expect(errorEntries).toHaveCount(3);
  });

  test('file names appear in completed list', async ({ page }) => {
    await importFiles(page, ['alpha.pdf', 'beta.pdf']);
    await selectAndConvert(page, ['alpha.pdf', 'beta.pdf']);

    await expect(page.getByText('Completed')).toBeVisible();
    const completedSection = page.locator('text=Completed >> ..');
    await expect(completedSection.getByText('alpha.pdf')).toBeVisible();
    await expect(completedSection.getByText('beta.pdf')).toBeVisible();
  });
});

// --- Cancellation (uses mocked hanging conversion) ---

test.describe('Cancellation', () => {
  test.beforeEach(async ({ page }) => {
    await mockHangingConversion(page);
    await page.goto('/import');
  });

  test('shows Cancel all button during active conversion', async ({
    page,
  }) => {
    await importFile(page, 'document.pdf');
    await selectAndConvert(page, ['document.pdf']);

    await expect(page).toHaveURL(/\/converting/);
    await expect(
      page.getByRole('button', { name: /cancel all/i }),
    ).toBeVisible();
  });

  test('opens confirmation dialog when Cancel all is clicked', async ({
    page,
  }) => {
    await importFile(page, 'document.pdf');
    await selectAndConvert(page, ['document.pdf']);

    await page.getByRole('button', { name: /cancel all/i }).click();

    const dialog = page.getByRole('dialog');
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText('Cancel conversions');
    await expect(dialog).toContainText('remaining conversion');
  });

  test('dismisses dialog when dialog Cancel button is clicked', async ({
    page,
  }) => {
    await importFile(page, 'document.pdf');
    await selectAndConvert(page, ['document.pdf']);

    await page.getByRole('button', { name: /cancel all/i }).click();
    const dialog = page.getByRole('dialog');
    await expect(dialog).toBeVisible();

    await dialog.getByRole('button', { name: 'Cancel' }).click();
    await expect(dialog).not.toBeVisible();

    await expect(
      page.getByRole('button', { name: /cancel all/i }),
    ).toBeVisible();
  });

  test('clears queue when cancellation is confirmed', async ({ page }) => {
    await importFiles(page, ['a.pdf', 'b.pdf', 'c.pdf']);
    await selectAndConvert(page, ['a.pdf', 'b.pdf', 'c.pdf']);

    await page.getByRole('button', { name: /cancel all/i }).click();
    const dialog = page.getByRole('dialog');
    await dialog.getByRole('button', { name: 'Confirm' }).click();
    await expect(dialog).not.toBeVisible();

    await expect(
      page.getByRole('button', { name: /cancel all/i }),
    ).not.toBeVisible();
  });

  test('dismisses dialog on Escape key', async ({ page }) => {
    await importFile(page, 'document.pdf');
    await selectAndConvert(page, ['document.pdf']);

    await page.getByRole('button', { name: /cancel all/i }).click();
    const dialog = page.getByRole('dialog');
    await expect(dialog).toBeVisible();

    await page.keyboard.press('Escape');
    await expect(dialog).not.toBeVisible();
  });

  test('confirmation message includes remaining count', async ({ page }) => {
    await importFiles(page, ['x.pdf', 'y.pdf', 'z.pdf']);
    await selectAndConvert(page, ['x.pdf', 'y.pdf', 'z.pdf']);

    await page.getByRole('button', { name: /cancel all/i }).click();
    const dialog = page.getByRole('dialog');
    await expect(dialog).toContainText(/Cancel \d+ remaining conversion/);
  });
});

// --- Queue display (uses mocked hanging conversion) ---

test.describe('Queue display', () => {
  test.beforeEach(async ({ page }) => {
    await mockHangingConversion(page);
    await page.goto('/import');
  });

  test('shows active file with Converting badge', async ({ page }) => {
    await importFile(page, 'active.pdf');
    await selectAndConvert(page, ['active.pdf']);

    await expect(page).toHaveURL(/\/converting/);
    await expect(
      page.locator('[aria-label="Status: Converting"]'),
    ).toBeVisible();
  });

  test('shows queued files with Queued label', async ({ page }) => {
    await importFiles(page, ['first.pdf', 'second.pdf', 'third.pdf']);
    await selectAndConvert(page, ['first.pdf', 'second.pdf', 'third.pdf']);

    await expect(page).toHaveURL(/\/converting/);
    const queuedLabels = page.getByText('Queued');
    await expect(queuedLabels).toHaveCount(2);
  });

  test('active file has a progress bar', async ({ page }) => {
    await importFile(page, 'doc.pdf');
    await selectAndConvert(page, ['doc.pdf']);

    await expect(page.getByRole('progressbar')).toBeVisible();
  });
});

// --- Progress bar accessibility (uses mocked hanging conversion) ---

test.describe('Progress bar accessibility', () => {
  test.beforeEach(async ({ page }) => {
    await mockHangingConversion(page);
    await page.goto('/import');
  });

  test('progress bar has correct ARIA attributes', async ({ page }) => {
    await importFile(page, 'doc.pdf');
    await selectAndConvert(page, ['doc.pdf']);

    const progressBar = page.getByRole('progressbar');
    await expect(progressBar).toBeVisible();
    await expect(progressBar).toHaveAttribute('aria-valuenow', '0');
    await expect(progressBar).toHaveAttribute('aria-valuemin', '0');
    await expect(progressBar).toHaveAttribute('aria-valuemax', '100');
    await expect(progressBar).toHaveAttribute(
      'aria-label',
      /Converting doc\.pdf/,
    );
  });
});

// --- Navigation from converting screen ---

test.describe('Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/import');
  });

  test('View converted button navigates to /converted', async ({ page }) => {
    await importFile(page, 'nav-test.pdf');
    await selectAndConvert(page, ['nav-test.pdf']);

    await expect(
      page.getByRole('button', { name: /view converted/i }),
    ).toBeVisible();
    await page.getByRole('button', { name: /view converted/i }).click();
    await expect(page).toHaveURL(/\/converted/);
  });

  test('clicking completed file row navigates to /converted', async ({
    page,
  }) => {
    await importFile(page, 'clickable.pdf');
    await selectAndConvert(page, ['clickable.pdf']);

    await expect(page.getByText('Completed')).toBeVisible();
    await page
      .locator('.cursor-pointer')
      .filter({ hasText: 'clickable.pdf' })
      .click();
    await expect(page).toHaveURL(/\/converted/);
  });
});
