import { test, expect, Page } from '@playwright/test';

/**
 * E2E tests for FlagLite Dashboard environments.
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

// Helper to navigate to environments page for a project
async function goToEnvironmentsPage(page: Page, projectName: string) {
  await page.goto('/projects');
  await expect(page.getByRole('heading', { name: /projects/i })).toBeVisible();

  // Click on the project card to go to flags page first
  await page.locator('.grid button').filter({ hasText: projectName }).click();
  await expect(page).toHaveURL(/\/projects\/[^/]+\/flags/);

  // Navigate to environments page via sidebar
  await page.getByRole('link', { name: /environments/i }).click();
  await expect(page).toHaveURL(/\/projects\/[^/]+\/environments/);
  await expect(page.getByRole('heading', { name: /environments/i })).toBeVisible();
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

// Helper to select environment tab on flags page
async function selectEnvironment(page: Page, envName: string) {
  await page.getByRole('button', { name: new RegExp(`^${envName}$`, 'i') }).click();
  // Wait for tab to be selected (has the active styling)
  await expect(
    page.getByRole('button', { name: new RegExp(`^${envName}$`, 'i') })
  ).toHaveClass(/border-green-600/);
}

// Helper to get toggle state for a flag
async function getFlagToggleState(page: Page, flagKey: string): Promise<boolean> {
  const row = page.locator('tr').filter({ hasText: flagKey });
  const toggle = row.getByRole('switch');
  const state = await toggle.getAttribute('aria-checked');
  return state === 'true';
}

// Helper to toggle a flag
async function toggleFlag(page: Page, flagKey: string) {
  const row = page.locator('tr').filter({ hasText: flagKey });
  const toggle = row.getByRole('switch');
  await toggle.click();
  // Wait for toggle to complete
  await page.waitForTimeout(500);
}

test.describe('Environments', () => {
  test.beforeEach(async ({ page }) => {
    await clearAuthState(page);
  });

  test.describe('List Environments', () => {
    test('shows default environments (development, staging, production)', async ({ page }) => {
      const projectName = uniqueProjectName('envlist');

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to environments page
      await goToEnvironmentsPage(page, projectName);

      // Verify all three default environments are visible
      await expect(page.getByText('Development')).toBeVisible();
      await expect(page.getByText('Staging')).toBeVisible();
      await expect(page.getByText('Production')).toBeVisible();

      // Verify we have exactly 3 environment cards
      const envCards = page.locator('.grid > div').filter({ hasText: /API Key/ });
      await expect(envCards).toHaveCount(3);
    });

    test('shows correct project context in page subtitle', async ({ page }) => {
      const projectName = uniqueProjectName('envcontext');

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to environments page
      await goToEnvironmentsPage(page, projectName);

      // Verify project name is shown in subtitle
      await expect(
        page.getByText(new RegExp(`manage environments for ${projectName}`, 'i'))
      ).toBeVisible();
    });
  });

  test.describe('Environment Details', () => {
    test('shows environment details including API key', async ({ page }) => {
      const projectName = uniqueProjectName('envdetails');

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to environments page
      await goToEnvironmentsPage(page, projectName);

      // Check each environment card has API key section
      for (const envName of ['Development', 'Staging', 'Production']) {
        const envCard = page.locator('.grid > div').filter({ hasText: envName });
        
        // Verify API Key label is present
        await expect(envCard.getByText('API Key')).toBeVisible();
        
        // Verify API key code element is present (truncated with ...)
        await expect(envCard.locator('code')).toBeVisible();
        await expect(envCard.locator('code')).toContainText('...');
      }
    });

    test('shows creation date for each environment', async ({ page }) => {
      const projectName = uniqueProjectName('envdate');

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to environments page
      await goToEnvironmentsPage(page, projectName);

      // Check each environment card has creation date
      for (const envName of ['Development', 'Staging', 'Production']) {
        const envCard = page.locator('.grid > div').filter({ hasText: envName });
        
        // Verify "Created" text is present
        await expect(envCard.getByText(/created/i)).toBeVisible();
      }
    });
  });

  test.describe('Copy API Key', () => {
    test('can copy environment API key to clipboard', async ({ page, context }) => {
      const projectName = uniqueProjectName('envcopy');

      // Grant clipboard permissions
      await context.grantPermissions(['clipboard-read', 'clipboard-write']);

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to environments page
      await goToEnvironmentsPage(page, projectName);

      // Find the development environment card and its copy button
      const devCard = page.locator('.grid > div').filter({ hasText: 'Development' });
      const copyButton = devCard.getByRole('button');

      // Click copy button
      await copyButton.click();

      // Wait for the check icon to appear (indicates success)
      await expect(devCard.locator('svg.text-green-600')).toBeVisible({ timeout: 2000 });

      // Verify clipboard contains the API key (starts with expected format)
      const clipboardText = await page.evaluate(() => navigator.clipboard.readText());
      expect(clipboardText).toMatch(/^[a-zA-Z0-9_-]+$/);
      expect(clipboardText.length).toBeGreaterThan(20);
    });

    test('copy button shows success feedback and resets', async ({ page, context }) => {
      const projectName = uniqueProjectName('envcopyreset');

      // Grant clipboard permissions
      await context.grantPermissions(['clipboard-read', 'clipboard-write']);

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to environments page
      await goToEnvironmentsPage(page, projectName);

      // Find the staging environment card and its copy button
      const stagingCard = page.locator('.grid > div').filter({ hasText: 'Staging' });
      const copyButton = stagingCard.getByRole('button');

      // Click copy button
      await copyButton.click();

      // Wait for the check icon to appear
      await expect(stagingCard.locator('svg.text-green-600')).toBeVisible({ timeout: 2000 });

      // Wait for the check icon to disappear (resets after ~2 seconds)
      await expect(stagingCard.locator('svg.text-green-600')).not.toBeVisible({ timeout: 5000 });
    });
  });

  test.describe('Environment Tabs on Flags Page', () => {
    test('shows environment tabs on flags page', async ({ page }) => {
      const projectName = uniqueProjectName('envtabsshow');

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Verify all environment tabs are visible
      await expect(page.getByRole('button', { name: /development/i })).toBeVisible();
      await expect(page.getByRole('button', { name: /staging/i })).toBeVisible();
      await expect(page.getByRole('button', { name: /production/i })).toBeVisible();
    });

    test('development tab is selected by default', async ({ page }) => {
      const projectName = uniqueProjectName('envdefault');

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Verify development is selected by default (has green border)
      await expect(
        page.getByRole('button', { name: /development/i })
      ).toHaveClass(/border-green-600/);

      // Verify other tabs are not selected
      await expect(
        page.getByRole('button', { name: /staging/i })
      ).not.toHaveClass(/border-green-600/);
      await expect(
        page.getByRole('button', { name: /production/i })
      ).not.toHaveClass(/border-green-600/);
    });

    test('can switch between environment tabs', async ({ page }) => {
      const projectName = uniqueProjectName('envswitch');

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Switch to staging
      await selectEnvironment(page, 'Staging');
      await expect(
        page.getByRole('button', { name: /staging/i })
      ).toHaveClass(/border-green-600/);
      await expect(
        page.getByRole('button', { name: /development/i })
      ).not.toHaveClass(/border-green-600/);

      // Switch to production
      await selectEnvironment(page, 'Production');
      await expect(
        page.getByRole('button', { name: /production/i })
      ).toHaveClass(/border-green-600/);
      await expect(
        page.getByRole('button', { name: /staging/i })
      ).not.toHaveClass(/border-green-600/);

      // Switch back to development
      await selectEnvironment(page, 'Development');
      await expect(
        page.getByRole('button', { name: /development/i })
      ).toHaveClass(/border-green-600/);
    });
  });

  test.describe('Flag State Per Environment', () => {
    test('flag states are independent across environments', async ({ page }) => {
      const projectName = uniqueProjectName('envindep');
      const flagKey = uniqueFlagKey('indep');
      const flagName = 'Independent Flag';

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Create a flag
      await page.getByRole('button', { name: /create your first flag/i }).click();
      await page.getByPlaceholder('enable-new-feature').fill(flagKey);
      await page.getByPlaceholder('Enable New Feature').fill(flagName);
      await page.getByRole('button', { name: 'Create Flag' }).click();
      await expect(page.getByText(flagName)).toBeVisible({ timeout: 10000 });

      // Enable flag in development
      await selectEnvironment(page, 'Development');
      await toggleFlag(page, flagKey);
      expect(await getFlagToggleState(page, flagKey)).toBe(true);

      // Switch to staging - flag should be OFF
      await selectEnvironment(page, 'Staging');
      expect(await getFlagToggleState(page, flagKey)).toBe(false);

      // Switch to production - flag should be OFF
      await selectEnvironment(page, 'Production');
      expect(await getFlagToggleState(page, flagKey)).toBe(false);

      // Go back to development - flag should still be ON
      await selectEnvironment(page, 'Development');
      expect(await getFlagToggleState(page, flagKey)).toBe(true);
    });

    test('toggling in one environment does not affect others', async ({ page }) => {
      const projectName = uniqueProjectName('envtoggle');
      const flagKey = uniqueFlagKey('toggle');
      const flagName = 'Toggle Test Flag';

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Create a flag
      await page.getByRole('button', { name: /create your first flag/i }).click();
      await page.getByPlaceholder('enable-new-feature').fill(flagKey);
      await page.getByPlaceholder('Enable New Feature').fill(flagName);
      await page.getByRole('button', { name: 'Create Flag' }).click();
      await expect(page.getByText(flagName)).toBeVisible({ timeout: 10000 });

      // Enable in development
      await selectEnvironment(page, 'Development');
      await toggleFlag(page, flagKey);
      expect(await getFlagToggleState(page, flagKey)).toBe(true);

      // Enable in staging
      await selectEnvironment(page, 'Staging');
      await toggleFlag(page, flagKey);
      expect(await getFlagToggleState(page, flagKey)).toBe(true);

      // Leave production disabled
      await selectEnvironment(page, 'Production');
      expect(await getFlagToggleState(page, flagKey)).toBe(false);

      // Disable staging
      await selectEnvironment(page, 'Staging');
      await toggleFlag(page, flagKey);
      expect(await getFlagToggleState(page, flagKey)).toBe(false);

      // Verify development is still enabled
      await selectEnvironment(page, 'Development');
      expect(await getFlagToggleState(page, flagKey)).toBe(true);

      // Verify production is still disabled
      await selectEnvironment(page, 'Production');
      expect(await getFlagToggleState(page, flagKey)).toBe(false);
    });

    test('flag state persists after page refresh', async ({ page }) => {
      const projectName = uniqueProjectName('envpersist');
      const flagKey = uniqueFlagKey('persist');
      const flagName = 'Persist Flag';

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Create a flag
      await page.getByRole('button', { name: /create your first flag/i }).click();
      await page.getByPlaceholder('enable-new-feature').fill(flagKey);
      await page.getByPlaceholder('Enable New Feature').fill(flagName);
      await page.getByRole('button', { name: 'Create Flag' }).click();
      await expect(page.getByText(flagName)).toBeVisible({ timeout: 10000 });

      // Enable in development
      await selectEnvironment(page, 'Development');
      await toggleFlag(page, flagKey);
      expect(await getFlagToggleState(page, flagKey)).toBe(true);

      // Enable in production
      await selectEnvironment(page, 'Production');
      await toggleFlag(page, flagKey);
      expect(await getFlagToggleState(page, flagKey)).toBe(true);

      // Refresh the page
      await page.reload();

      // Wait for page to load
      await expect(page.getByText(flagName)).toBeVisible({ timeout: 10000 });

      // Verify development is still ON
      await selectEnvironment(page, 'Development');
      expect(await getFlagToggleState(page, flagKey)).toBe(true);

      // Verify staging is still OFF
      await selectEnvironment(page, 'Staging');
      expect(await getFlagToggleState(page, flagKey)).toBe(false);

      // Verify production is still ON
      await selectEnvironment(page, 'Production');
      expect(await getFlagToggleState(page, flagKey)).toBe(true);
    });
  });
});
