import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for FlagLite Dashboard E2E tests.
 * 
 * Run tests:
 *   npm run test:e2e          # headless
 *   npm run test:e2e:ui       # with UI mode
 * 
 * Prerequisites:
 *   - Dashboard dev server running: npm run dev
 *   - API server running (via docker-compose.e2e.yml or locally)
 */
export default defineConfig({
  testDir: './e2e',
  
  // Run tests in parallel
  fullyParallel: true,
  
  // Fail the build on CI if you accidentally left test.only in the source code
  forbidOnly: !!process.env.CI,
  
  // Retry on CI only
  retries: process.env.CI ? 2 : 0,
  
  // Opt out of parallel tests on CI (more stable)
  workers: process.env.CI ? 1 : undefined,
  
  // Reporter configuration
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['list'],
  ],
  
  // Shared settings for all projects
  use: {
    // Base URL for navigation
    baseURL: process.env.PLAYWRIGHT_BASE_URL ?? 'http://localhost:3001',
    
    // Capture screenshot on failure
    screenshot: 'only-on-failure',
    
    // Record trace on first retry
    trace: 'on-first-retry',
    
    // Record video on failure
    video: 'on-first-retry',
  },

  // Configure projects for major browsers
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    // Uncomment to test in additional browsers:
    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },
    // {
    //   name: 'webkit',
    //   use: { ...devices['Desktop Safari'] },
    // },
  ],

  // Run local dev server before starting tests (only when not in CI)
  // Uncomment if you want Playwright to start the dev server automatically:
  // webServer: {
  //   command: 'npm run dev',
  //   url: 'http://localhost:5173',
  //   reuseExistingServer: !process.env.CI,
  //   timeout: 120 * 1000,
  // },
});
