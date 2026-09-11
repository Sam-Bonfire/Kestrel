import { test, expect } from '@playwright/test';

test.describe('calendar', () => {
  test('loads behind the login gate', async ({ page }) => {
    await page.goto('/');
    await page.waitForURL('**/login');
    await expect(page.getByPlaceholder('username@kestrel.dev')).toBeVisible();
  });

  test('failed sign-in surfaces an error banner', async ({ page }) => {
    await page.goto('/login');
    await page.getByPlaceholder('username@kestrel.dev').fill('nobody@kestrel.dev');
    await page.locator('#password').fill('wrong-password-123');
    await page.getByRole('button', { name: 'Sign In' }).click();
    await expect(page.getByRole('alert')).toContainText('Cannot reach the server', { timeout: 15000 });
  });
});
