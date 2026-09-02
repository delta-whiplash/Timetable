import { defineConfig, devices } from '@playwright/test';

/**
 * Configuration Playwright pour Timetable Desktop E2E tests.
 *
 * Pour exécuter les tests:
 * - Terminal 1: `npm run dev` (lance le serveur Vite sur localhost:1420)
 * - Terminal 2: `npm run test:e2e`
 *
 * Note: Ces tests ciblent l'application en mode dev web.
 * Pour tester l'app Tauri empaquetée, une configuration différente serait nécessaire.
 */
export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  reporter: 'html',

  use: {
    baseURL: 'http://localhost:1420',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      // PLAYWRIGHT_EXECUTABLE / PLAYWRIGHT_CHANNEL let a machine reuse an
      // existing browser instead of downloading one (e.g. system Edge).
      use: {
        ...devices['Desktop Chrome'],
        ...(process.env.PLAYWRIGHT_CHANNEL ? { channel: process.env.PLAYWRIGHT_CHANNEL } : {}),
        ...(process.env.PLAYWRIGHT_EXECUTABLE
          ? { launchOptions: { executablePath: process.env.PLAYWRIGHT_EXECUTABLE } }
          : {}),
      },
    },
  ],

  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
