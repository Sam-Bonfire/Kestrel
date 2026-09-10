import { defineConfig, devices } from '@playwright/test';

const MAIL_PORT = Number(process.env.E2E_MAIL_PORT ?? 1423);
const CAL_PORT = Number(process.env.E2E_CAL_PORT ?? 1424);

/**
 * Offline-first smoke suite: built frontends served over vite preview,
 * no backend required. Backend-backed flows are follow-ups.
 */
export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  retries: process.env.CI ? 2 : 0,
  reporter: 'list',
  use: {
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'mail',
      testMatch: /mail\.spec\.ts/,
      use: { ...devices['Desktop Chrome'], baseURL: `http://localhost:${MAIL_PORT}` },
    },
    {
      name: 'calendar',
      testMatch: /calendar\.spec\.ts/,
      use: { ...devices['Desktop Chrome'], baseURL: `http://localhost:${CAL_PORT}` },
    },
  ],
  webServer: [
    {
      command: `pnpm --filter frontend-mail exec vite preview --port ${MAIL_PORT} --strictPort`,
      url: `http://localhost:${MAIL_PORT}`,
      reuseExistingServer: !process.env.CI,
      cwd: '..',
      timeout: 120000,
    },
    {
      command: `pnpm --filter frontend-calendar exec vite preview --port ${CAL_PORT} --strictPort`,
      url: `http://localhost:${CAL_PORT}`,
      reuseExistingServer: !process.env.CI,
      cwd: '..',
      timeout: 120000,
    },
  ],
});
