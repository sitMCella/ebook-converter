import { test, expect } from '@playwright/test';

test.describe('Import Screen — Layout', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/import');
  });

  test('renders the page heading and browse button', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Import PDF files' })).toBeVisible();
    await expect(page.getByRole('button', { name: /browse files/i })).toBeVisible();
  });

  test('renders the drop zone with instructions', async ({ page }) => {
    const dropZone = page.getByRole('button', { name: /drop zone for pdf files/i });
    await expect(dropZone).toBeVisible();
    await expect(page.getByText('Drop PDF files here')).toBeVisible();
    await expect(page.getByText(/click.*browse files.*to select/i)).toBeVisible();
  });

  test('shows empty state when no files staged', async ({ page }) => {
    await expect(page.getByText('No files staged yet.')).toBeVisible();
  });

  test('sidebar highlights Import as active nav item', async ({ page }) => {
    const importLink = page.locator('nav a[href="/import"]');
    await expect(importLink).toHaveClass(/font-medium/);
    await expect(importLink).toHaveClass(/text-\[var\(--text-accent\)\]/);
  });

  test('sidebar shows all navigation items', async ({ page }) => {
    await expect(page.locator('nav').getByText('Import')).toBeVisible();
    await expect(page.locator('nav').getByText('Library')).toBeVisible();
    await expect(page.locator('nav').getByText('Converted')).toBeVisible();
    await expect(page.locator('nav').getByText('Settings')).toBeVisible();
    await expect(page.locator('nav').getByText('Tools')).toBeVisible();
  });

  test('batch action buttons are present and disabled initially', async ({ page }) => {
    const removeBtn = page.getByRole('button', { name: /remove selected/i });
    const importBtn = page.getByRole('button', { name: /import to library/i });
    await expect(removeBtn).toBeVisible();
    await expect(importBtn).toBeVisible();
    await expect(removeBtn).toBeDisabled();
    await expect(importBtn).toBeDisabled();
  });
});

test.describe('Import Screen — File Staging via Browse', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/import');
  });

  test('browse button triggers file input and stages a PDF', async ({ page }) => {
    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.getByRole('button', { name: /browse files/i }).click();
    const fileChooser = await fileChooserPromise;

    expect(fileChooser.isMultiple()).toBe(true);

    await fileChooser.setFiles(createFakeFile('test-document.pdf'));

    await expect(page.getByText('test-document.pdf')).toBeVisible();
    await expect(page.getByText('No files staged yet.')).not.toBeVisible();
  });

  test('stages multiple PDF files at once', async ({ page }) => {
    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.getByRole('button', { name: /browse files/i }).click();
    const fileChooser = await fileChooserPromise;

    await fileChooser.setFiles([
      createFakeFile('first.pdf'),
      createFakeFile('second.pdf'),
      createFakeFile('third.pdf'),
    ]);

    await expect(page.getByText('first.pdf')).toBeVisible();
    await expect(page.getByText('second.pdf')).toBeVisible();
    await expect(page.getByText('third.pdf')).toBeVisible();
  });

  test('shows "Ready to import" label when files are staged', async ({ page }) => {
    await stageFile(page, 'document.pdf');
    await expect(page.getByText('Ready to import')).toBeVisible();
  });
});

test.describe('Import Screen — Duplicate Detection', () => {
  test('shows toast when staging a duplicate file', async ({ page }) => {
    await page.goto('/import');

    await stageFile(page, 'duplicate.pdf');
    await expect(page.getByText('duplicate.pdf')).toBeVisible();

    await stageFile(page, 'duplicate.pdf');
    await expect(page.getByText('File already imported')).toBeVisible();

    const rows = page.locator('[class*="border-b"]').filter({ hasText: 'duplicate.pdf' });
    await expect(rows).toHaveCount(1);
  });
});

test.describe('Import Screen — Selection and Batch Actions', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/import');
    await stageFile(page, 'file-a.pdf');
    await stageFile(page, 'file-b.pdf');
  });

  test('selecting a file enables "Remove selected" button', async ({ page }) => {
    const removeBtn = page.getByRole('button', { name: /remove selected/i });
    await expect(removeBtn).toBeDisabled();

    await page.getByRole('checkbox', { name: /select file-a\.pdf/i }).check();
    await expect(removeBtn).toBeEnabled();
  });

  test('selecting a ready file enables "Import to library" button', async ({ page }) => {
    const importBtn = page.getByRole('button', { name: /import to library/i });
    await expect(importBtn).toBeDisabled();

    await page.getByRole('checkbox', { name: /select file-a\.pdf/i }).check();
    await expect(importBtn).toBeEnabled();
  });

  test('remove selected unstages files immediately without confirmation', async ({ page }) => {
    await page.getByRole('checkbox', { name: /select file-a\.pdf/i }).check();
    await page.getByRole('button', { name: /remove selected/i }).click();

    await expect(page.getByText('file-a.pdf')).not.toBeVisible();
    await expect(page.getByText('file-b.pdf')).toBeVisible();
  });

  test('removing all files shows empty state again', async ({ page }) => {
    await page.getByRole('checkbox', { name: /select file-a\.pdf/i }).check();
    await page.getByRole('checkbox', { name: /select file-b\.pdf/i }).check();
    await page.getByRole('button', { name: /remove selected/i }).click();

    await expect(page.getByText('No files staged yet.')).toBeVisible();
  });

  test('import to library moves files from staging to library', async ({ page }) => {
    await page.getByRole('checkbox', { name: /select file-a\.pdf/i }).check();
    await page.getByRole('button', { name: /import to library/i }).click();

    await expect(page.getByText('file-a.pdf')).not.toBeVisible();
    await expect(page.getByText('file-b.pdf')).toBeVisible();

    await page.locator('nav').getByText('Library').click();
    await expect(page).toHaveURL(/\/library/);
    const listbox = page.getByRole('listbox', { name: /document list/i });
    await expect(listbox.getByText('file-a.pdf')).toBeVisible();
  });
});

test.describe('Import Screen — Drop Zone Interaction', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/import');
  });

  test('drop zone is focusable and keyboard-activatable', async ({ page }) => {
    const dropZone = page.getByRole('button', { name: /drop zone for pdf files/i });
    await dropZone.focus();
    await expect(dropZone).toBeFocused();

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.keyboard.press('Enter');
    const fileChooser = await fileChooserPromise;
    expect(fileChooser).toBeTruthy();
  });

  test('drop zone activates on Space key', async ({ page }) => {
    const dropZone = page.getByRole('button', { name: /drop zone for pdf files/i });
    await dropZone.focus();

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.keyboard.press('Space');
    const fileChooser = await fileChooserPromise;
    expect(fileChooser).toBeTruthy();
  });
});

test.describe('Import Screen — Keyboard Shortcuts', () => {
  test('Cmd/Ctrl+O triggers file dialog', async ({ page }) => {
    await page.goto('/import');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.evaluate(() => {
      window.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'o', metaKey: true, bubbles: true })
      );
    });
    const fileChooser = await fileChooserPromise;
    expect(fileChooser).toBeTruthy();
  });
});

test.describe('Import Screen — Sidebar Navigation', () => {
  test('navigating to Library via sidebar updates active state', async ({ page }) => {
    await page.goto('/import');

    await page.locator('nav').getByText('Library').click();
    await expect(page).toHaveURL(/\/library/);

    const libraryLink = page.locator('nav a[href="/library"]');
    await expect(libraryLink).toHaveClass(/font-medium/);
  });

  test('navigating back to Import via sidebar retains staged files', async ({ page }) => {
    await page.goto('/import');
    await stageFile(page, 'persist-test.pdf');

    await page.locator('nav').getByText('Library').click();
    await page.locator('nav').getByText('Import').click();

    await expect(page.getByText('persist-test.pdf')).toBeVisible();
  });
});

test.describe('Import Screen — Status Badges', () => {
  test('does not show status badge for non-error files', async ({ page }) => {
    await page.goto('/import');
    await stageFile(page, 'accessible.pdf');

    const badge = page.locator('[aria-label="Status: Ready"]');
    await expect(badge).not.toBeVisible();
  });
});

test.describe('Import Screen — Root Redirect', () => {
  test('navigating to / redirects to /import', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveURL(/\/import/);
    await expect(page.getByRole('heading', { name: 'Import PDF files' })).toBeVisible();
  });
});

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
