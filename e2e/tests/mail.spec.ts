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
