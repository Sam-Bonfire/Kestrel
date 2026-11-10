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

const BACKEND_MODE = !!process.env.E2E_BACKEND;
const SEED_USER = process.env.KESTREL_E2E_USER ?? 'demo_user';
// Must match KESTREL_SEED_PASSWORD; the runner defaults both to this value.
const SEED_PASSWORD = process.env.KESTREL_E2E_PASSWORD ?? 'e2e-seed-password';

test.describe('mail @backend live-server suite', () => {
  test.skip(!BACKEND_MODE, 'runs only in backend mode: mise run test:e2e:backend (E2E_BACKEND=1)');

  async function loginSeededUser(page: import('@playwright/test').Page): Promise<void> {
    await page.goto('/login');
    await page.getByPlaceholder('username@kestrel.dev').fill(SEED_USER);
    await page.locator('#password').fill(SEED_PASSWORD);
    await page.getByRole('button', { name: 'Sign In' }).click();
    await expect(page).not.toHaveURL(/\/login/);
  }

  test('logs in and renders seeded mail from the real backend', async ({ page }) => {
    await loginSeededUser(page);
    await expect(page.getByText('E2E Integration Wirecheck').first()).toBeVisible({ timeout: 20000 });
    await expect(page.getByText('E2E Fixture Second Inbox Item').first()).toBeVisible();
  });

  test('pre-archived seeded mail does not appear in the inbox', async ({ page }) => {
    await loginSeededUser(page);
    await expect(page.getByText('E2E Integration Wirecheck').first()).toBeVisible({ timeout: 20000 });
    await expect(page.getByText('E2E Fixture Archived Probe')).toHaveCount(0);
  });

  test('pressing E archives the selected message', async ({ page }) => {
    await loginSeededUser(page);
    const subject = page.getByText('E2E Fixture Second Inbox Item').first();
    await expect(subject).toBeVisible({ timeout: 20000 });
    await subject.click();
    await page.keyboard.press('e');
    await expect(subject).toHaveCount(0, { timeout: 15000 });
  });
});
