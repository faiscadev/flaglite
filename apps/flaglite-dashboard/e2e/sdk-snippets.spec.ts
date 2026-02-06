import { test, expect, Page } from '@playwright/test';

/**
 * E2E tests for FlagLite Dashboard SDK Snippets feature.
 *
 * Prerequisites:
 *   - docker compose -f docker-compose.e2e.yml up -d
 *   - npm run dev (dashboard on port 3001)
 */

// Generate unique username for each test run to ensure independence
function uniqueUsername(prefix = 'e2e') {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
}

// Generate unique project name
function uniqueProjectName(prefix = 'proj') {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
}

// Generate unique flag key
function uniqueFlagKey(prefix = 'flag') {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
}

// Helper to clear auth state before tests
async function clearAuthState(page: Page) {
  await page.goto('/login');
  await page.evaluate(() => {
    localStorage.clear();
    sessionStorage.clear();
  });
}

// Helper to verify user is logged in (projects page with navigation)
async function expectLoggedIn(page: Page) {
  await expect(page).toHaveURL(/\/projects/);
  await expect(page.getByRole('heading', { name: /projects/i })).toBeVisible();
}

// Helper to signup a new user and return credentials
async function signupUser(
  page: Page,
  username?: string,
  password = 'testpass123',
  projectName?: string
) {
  const user = username || uniqueUsername();

  await page.goto('/signup');
  await page.getByPlaceholder(/cool-developer/i).fill(user);
  await page.getByPlaceholder(/••••••••/).fill(password);

  // Optionally fill project name
  if (projectName) {
    await page.getByPlaceholder(/my-awesome-app/i).fill(projectName);
  }

  await page.getByRole('button', { name: /sign up/i }).click();

  // Wait for API key screen
  await expect(
    page.getByRole('heading', { name: /account created/i })
  ).toBeVisible({ timeout: 10000 });

  // Click through the API key confirmation
  await page.getByRole('button', { name: /i've saved my api key/i }).click();

  // Wait for redirect to projects
  await expectLoggedIn(page);

  return { username: user, password };
}

// Helper to navigate to flags page for a project
async function goToFlagsPage(page: Page, projectName: string) {
  await page.goto('/projects');
  await expect(page.getByRole('heading', { name: /projects/i })).toBeVisible();

  // Click on the project card to go to flags page
  await page.locator('.grid button').filter({ hasText: projectName }).click();
  await expect(page).toHaveURL(/\/projects\/[^/]+\/flags/);
  await expect(page.getByRole('heading', { name: /flags/i })).toBeVisible();
}

// Helper to create a flag and navigate to its detail page
async function createFlagAndGoToDetail(
  page: Page,
  projectName: string,
  flagKey: string,
  flagName: string
) {
  // Go to flags page
  await goToFlagsPage(page, projectName);

  // Create a flag
  await page.getByRole('button', { name: /create your first flag/i }).click();
  await page.getByPlaceholder('enable-new-feature').fill(flagKey);
  await page.getByPlaceholder('Enable New Feature').fill(flagName);
  await page.getByRole('button', { name: 'Create Flag' }).click();
  await expect(page.getByText(flagName)).toBeVisible({ timeout: 10000 });

  // Navigate to detail page
  await page.getByText(flagName).click();
  await expect(page).toHaveURL(new RegExp(`/flags/${flagKey}$`));
  await expect(page.getByRole('heading', { name: flagName })).toBeVisible();
}

test.describe('SDK Snippets', () => {
  test.beforeEach(async ({ page }) => {
    await clearAuthState(page);
  });

  test('SDK integration section is visible on flag detail page', async ({
    page,
  }) => {
    const projectName = uniqueProjectName('sdkvisible');
    const flagKey = uniqueFlagKey('sdkvisible');
    const flagName = 'SDK Visibility Test';

    await signupUser(page, undefined, 'testpass123', projectName);
    await createFlagAndGoToDetail(page, projectName, flagKey, flagName);

    // Verify SDK Integration section is visible
    await expect(
      page.getByRole('heading', { name: /sdk integration/i })
    ).toBeVisible();

    // Verify code snippet area exists
    await expect(page.locator('pre code')).toBeVisible();
  });

  test('all language tabs are present (JavaScript, Python, Go, Rust)', async ({
    page,
  }) => {
    const projectName = uniqueProjectName('sdktabs');
    const flagKey = uniqueFlagKey('sdktabs');
    const flagName = 'SDK Tabs Test';

    await signupUser(page, undefined, 'testpass123', projectName);
    await createFlagAndGoToDetail(page, projectName, flagKey, flagName);

    // Verify all language tabs are present
    await expect(page.getByRole('button', { name: 'JavaScript' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Python' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Go', exact: true })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Rust' })).toBeVisible();
  });

  test('JavaScript snippet shows correct flag key and syntax', async ({
    page,
  }) => {
    const projectName = uniqueProjectName('sdkjs');
    const flagKey = uniqueFlagKey('sdkjs');
    const flagName = 'SDK JavaScript Test';

    await signupUser(page, undefined, 'testpass123', projectName);
    await createFlagAndGoToDetail(page, projectName, flagKey, flagName);

    // Click JavaScript tab (should be default, but click to be sure)
    await page.getByRole('button', { name: 'JavaScript' }).click();

    // Verify JavaScript-specific syntax in snippet
    const codeBlock = page.locator('pre code');
    await expect(codeBlock).toContainText(flagKey);
    await expect(codeBlock).toContainText('import { FlagLiteClient }');
    await expect(codeBlock).toContainText("from '@flaglite/sdk'");
    await expect(codeBlock).toContainText('client.isEnabled');
    await expect(codeBlock).toContainText('await');
  });

  test('Python snippet shows correct flag key and syntax', async ({ page }) => {
    const projectName = uniqueProjectName('sdkpy');
    const flagKey = uniqueFlagKey('sdkpy');
    const flagName = 'SDK Python Test';

    await signupUser(page, undefined, 'testpass123', projectName);
    await createFlagAndGoToDetail(page, projectName, flagKey, flagName);

    // Click Python tab
    await page.getByRole('button', { name: 'Python' }).click();

    // Verify Python-specific syntax in snippet
    const codeBlock = page.locator('pre code');
    await expect(codeBlock).toContainText(flagKey);
    await expect(codeBlock).toContainText('from flaglite import FlagLiteClient');
    await expect(codeBlock).toContainText('client.is_enabled');
    await expect(codeBlock).toContainText('api_key=');
  });

  test('Go snippet shows correct flag key and syntax', async ({ page }) => {
    const projectName = uniqueProjectName('sdkgo');
    const flagKey = uniqueFlagKey('sdkgo');
    const flagName = 'SDK Go Test';

    await signupUser(page, undefined, 'testpass123', projectName);
    await createFlagAndGoToDetail(page, projectName, flagKey, flagName);

    // Click Go tab (use exact: true to avoid conflict with project card)
    await page.getByRole('button', { name: 'Go', exact: true }).click();

    // Verify Go-specific syntax in snippet
    const codeBlock = page.locator('pre code');
    await expect(codeBlock).toContainText(flagKey);
    await expect(codeBlock).toContainText('package main');
    await expect(codeBlock).toContainText('import (');
    await expect(codeBlock).toContainText('github.com/faiscadev/flaglite-go');
    await expect(codeBlock).toContainText('client.IsEnabled');
    await expect(codeBlock).toContainText('flaglite.NewClient');
  });

  test('Rust snippet shows correct flag key and syntax', async ({ page }) => {
    const projectName = uniqueProjectName('sdkrs');
    const flagKey = uniqueFlagKey('sdkrs');
    const flagName = 'SDK Rust Test';

    await signupUser(page, undefined, 'testpass123', projectName);
    await createFlagAndGoToDetail(page, projectName, flagKey, flagName);

    // Click Rust tab
    await page.getByRole('button', { name: 'Rust' }).click();

    // Verify Rust-specific syntax in snippet
    const codeBlock = page.locator('pre code');
    await expect(codeBlock).toContainText(flagKey);
    await expect(codeBlock).toContainText('use flaglite::FlagLiteClient');
    await expect(codeBlock).toContainText('#[tokio::main]');
    await expect(codeBlock).toContainText('async fn main');
    await expect(codeBlock).toContainText('client.is_enabled');
  });

  test('copy snippet button works', async ({ page, context }) => {
    const projectName = uniqueProjectName('sdkcopy');
    const flagKey = uniqueFlagKey('sdkcopy');
    const flagName = 'SDK Copy Test';

    // Grant clipboard permissions
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);

    await signupUser(page, undefined, 'testpass123', projectName);
    await createFlagAndGoToDetail(page, projectName, flagKey, flagName);

    // Get the code snippet content before clicking
    const codeContent = await page.locator('pre code').textContent();

    // Find and click the copy button (inside the SDK Integration section's pre block)
    const sdkSection = page.locator('.bg-white').filter({
      has: page.getByRole('heading', { name: /sdk integration/i }),
    });
    const copyButton = sdkSection.locator('button').filter({
      has: page.locator('svg'),
    });
    await copyButton.click();

    // Wait for copy confirmation (Check icon should appear)
    await expect(copyButton.locator('svg.text-green-400')).toBeVisible({
      timeout: 2000,
    });

    // Verify clipboard content contains the flag key
    const clipboardContent = await page.evaluate(() =>
      navigator.clipboard.readText()
    );
    expect(clipboardContent).toContain(flagKey);
    expect(clipboardContent).toContain('FlagLiteClient');
  });

  test('switching language tabs updates the code snippet', async ({ page }) => {
    const projectName = uniqueProjectName('sdkswitch');
    const flagKey = uniqueFlagKey('sdkswitch');
    const flagName = 'SDK Switch Test';

    await signupUser(page, undefined, 'testpass123', projectName);
    await createFlagAndGoToDetail(page, projectName, flagKey, flagName);

    const codeBlock = page.locator('pre code');

    // Start with JavaScript (default)
    await page.getByRole('button', { name: 'JavaScript' }).click();
    await expect(codeBlock).toContainText("from '@flaglite/sdk'");

    // Switch to Python
    await page.getByRole('button', { name: 'Python' }).click();
    await expect(codeBlock).toContainText('from flaglite import');
    await expect(codeBlock).not.toContainText("from '@flaglite/sdk'");

    // Switch to Go
    await page.getByRole('button', { name: 'Go', exact: true }).click();
    await expect(codeBlock).toContainText('package main');
    await expect(codeBlock).not.toContainText('from flaglite import');

    // Switch to Rust
    await page.getByRole('button', { name: 'Rust' }).click();
    await expect(codeBlock).toContainText('use flaglite::FlagLiteClient');
    await expect(codeBlock).not.toContainText('package main');

    // Switch back to JavaScript
    await page.getByRole('button', { name: 'JavaScript' }).click();
    await expect(codeBlock).toContainText("from '@flaglite/sdk'");
  });

  test('selected language tab has active styling', async ({ page }) => {
    const projectName = uniqueProjectName('sdkstyle');
    const flagKey = uniqueFlagKey('sdkstyle');
    const flagName = 'SDK Style Test';

    await signupUser(page, undefined, 'testpass123', projectName);
    await createFlagAndGoToDetail(page, projectName, flagKey, flagName);

    // JavaScript should be selected by default (has active styling)
    await expect(page.getByRole('button', { name: 'JavaScript' })).toHaveClass(
      /bg-green-100/
    );
    await expect(page.getByRole('button', { name: 'Python' })).not.toHaveClass(
      /bg-green-100/
    );

    // Click Python - should now have active styling
    await page.getByRole('button', { name: 'Python' }).click();
    await expect(page.getByRole('button', { name: 'Python' })).toHaveClass(
      /bg-green-100/
    );
    await expect(
      page.getByRole('button', { name: 'JavaScript' })
    ).not.toHaveClass(/bg-green-100/);

    // Click Go
    await page.getByRole('button', { name: 'Go', exact: true }).click();
    await expect(page.getByRole('button', { name: 'Go', exact: true })).toHaveClass(
      /bg-green-100/
    );
    await expect(page.getByRole('button', { name: 'Python' })).not.toHaveClass(
      /bg-green-100/
    );

    // Click Rust
    await page.getByRole('button', { name: 'Rust' }).click();
    await expect(page.getByRole('button', { name: 'Rust' })).toHaveClass(
      /bg-green-100/
    );
    await expect(page.getByRole('button', { name: 'Go', exact: true })).not.toHaveClass(
      /bg-green-100/
    );
  });
});
