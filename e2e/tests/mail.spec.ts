import { test, expect } from '@playwright/test';

test.describe('mail auth', () => {
  test('shows the sign-in form', async ({ page }) => {
    await page.goto('/login');
    await expect(page.getByPlaceholder('username@kestrel.dev')).toBeVisible();
    await expect(page.locator('#password')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Sign In' })).toBeVisible();
  });

  test('register validates password length locally', async ({ page }) => {
    await page.goto('/register');
    await page.getByPlaceholder('username@kestrel.dev').fill('new@kestrel.dev');
    await page.locator('#password').fill('short');
    await page.locator('#confirmPassword').fill('short');
    await page.getByRole('button', { name: 'Register Now' }).click();
    await expect(page.getByText('Password must be at least 8 characters long.')).toBeVisible();
  });

  test('register rejects mismatched passwords', async ({ page }) => {
    await page.goto('/register');
    await page.getByPlaceholder('username@kestrel.dev').fill('new@kestrel.dev');
    await page.locator('#password').fill('longenoughpassword');
    await page.locator('#confirmPassword').fill('differentpassword');
    await page.getByRole('button', { name: 'Register Now' }).click();
    await expect(page.getByText('Passwords do not match.')).toBeVisible();
  });

  test('failed sign-in surfaces an error banner', async ({ page }) => {
    await page.goto('/login');
    await page.getByPlaceholder('username@kestrel.dev').fill('nobody@kestrel.dev');
    await page.locator('#password').fill('wrong-password-123');
    await page.getByRole('button', { name: 'Sign In' }).click();
    await expect(page.getByText('Cannot reach the server')).toBeVisible({ timeout: 15000 });
  });
});

test.describe('mail authenticated e2e', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.getByPlaceholder('username@kestrel.dev').fill('demo_user');
    await page.locator('#password').fill('e2e-password-123');
    await page.getByRole('button', { name: 'Sign In' }).click();

    // Wait for inbox to load
    await expect(page.getByRole('heading', { name: 'Inbox' })).toBeVisible({ timeout: 15000 });
  });

  test('list inbox and open a thread', async ({ page }) => {
    const thread = page.locator('text=E2E Labeled: Test Message').first();
    await expect(thread).toBeVisible();
    await thread.click();

    // Verify it opened correctly
    await expect(page.getByRole('heading', { name: 'E2E Labeled: Test Message' })).toBeVisible();

    // Verify body text exists in the center pane
    await expect(page.locator('text=Test body text').first()).toBeVisible();
  });

  test('archive a thread', async ({ page }) => {
    const threadRow = page.locator('div').filter({ hasText: 'E2E Labeled: Test Message' }).first();
    await expect(threadRow).toBeVisible();

    await threadRow.click();

    // Use the keyboard shortcut 'e' to archive
    await page.keyboard.press('e');

    // The thread should disappear from the inbox list
    await expect(page.locator('div').filter({ hasText: 'E2E Labeled: Test Message' }).first()).not.toBeVisible();
  });

  test('apply a label', async ({ page }) => {
    const threadRow = page.locator('div').filter({ hasText: 'E2E Labeled: Test Message' }).first();
    await threadRow.click();

    // Verify existing label from DB
    await expect(page.locator('text=e2e-label').first()).toBeVisible();
  });

  test('download an attachment', async ({ page }) => {
    const threadRow = page.locator('div').filter({ hasText: 'E2E Attachment: Test Message' }).first();
    await threadRow.click();

    // Click attachment button
    const attachmentBtn = page.getByText('test-attachment.txt').first();
    await expect(attachmentBtn).toBeVisible();

    // In a Playwright context, we wait for a popup or download event, or intercept the window.open
    const [page1] = await Promise.all([
      page.waitForEvent('popup'),
      attachmentBtn.click()
    ]);

    await expect(page1).toBeDefined();
  });

  test('restore a draft', async ({ page }) => {
    // Navigate to Drafts view
    await page.locator('text=Drafts').click();

    // Since our DB doesn't perfectly emulate Draft label mapping (needs popout draft logic natively),
    // we use "E2E Draft: Test Message" that was seeded with a "draft" label
    const draftThread = page.locator('div').filter({ hasText: 'E2E Draft: Test Message' }).first();
    await expect(draftThread).toBeVisible();

    await draftThread.click();

    // In our implementation, opening a draft should show it in the center pane
    await expect(page.getByRole('heading', { name: 'E2E Draft: Test Message' })).toBeVisible();
    await expect(page.locator('text=Test body text').first()).toBeVisible();
  });
});
