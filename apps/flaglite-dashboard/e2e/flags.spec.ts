import { test, expect, Page } from '@playwright/test';

/**
 * E2E tests for FlagLite Dashboard flags CRUD.
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

// Helper to select environment tab
async function selectEnvironment(page: Page, envName: string) {
  await page.getByRole('button', { name: new RegExp(`^${envName}$`, 'i') }).click();
  // Wait for tab to be selected (has the active styling)
  await expect(
    page.getByRole('button', { name: new RegExp(`^${envName}$`, 'i') })
  ).toHaveClass(/border-green-600/);
}

test.describe('Flags', () => {
  test.beforeEach(async ({ page }) => {
    await clearAuthState(page);
  });

  test.describe('List Flags', () => {
    test('shows empty state when no flags exist', async ({ page }) => {
      const projectName = uniqueProjectName('noflags');

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Verify empty state
      await expect(page.getByText(/no feature flags yet/i)).toBeVisible();
      await expect(
        page.getByRole('button', { name: /create your first flag/i })
      ).toBeVisible();
    });

    test('shows flags in table after creation', async ({ page }) => {
      const projectName = uniqueProjectName('withflags');
      const flagKey = uniqueFlagKey();
      const flagName = 'My Test Flag';

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Create a flag - inline to avoid helper issues
      await page.getByRole('button', { name: /create your first flag/i }).click();
      await page.getByPlaceholder('enable-new-feature').fill(flagKey);
      await page.getByPlaceholder('Enable New Feature').fill(flagName);
      await page.getByRole('button', { name: 'Create Flag' }).click();

      // Wait for flag to appear in the table
      await expect(page.getByText(flagName)).toBeVisible({ timeout: 10000 });
      await expect(page.locator('code').filter({ hasText: flagKey })).toBeVisible();
    });

    test('shows correct project context in page subtitle', async ({ page }) => {
      const projectName = uniqueProjectName('context');

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Verify project name is shown in subtitle
      await expect(
        page.getByText(new RegExp(`manage feature flags for ${projectName}`, 'i'))
      ).toBeVisible();
    });
  });

  test.describe('Create Flag', () => {
    test('can create boolean flag via modal', async ({ page }) => {
      const projectName = uniqueProjectName('createflag');
      const flagKey = uniqueFlagKey('new');
      const flagName = 'Enable New Feature';

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Create a flag
      await page.getByRole('button', { name: /create your first flag/i }).click();
      await page.getByPlaceholder('enable-new-feature').fill(flagKey);
      await page.getByPlaceholder('Enable New Feature').fill(flagName);
      await page.getByRole('button', { name: 'Create Flag' }).click();

      // Verify flag appears in the table
      await expect(page.getByText(flagName)).toBeVisible({ timeout: 10000 });
      await expect(page.locator('code').filter({ hasText: flagKey })).toBeVisible();
    });

    test('new flag is disabled by default', async ({ page }) => {
      const projectName = uniqueProjectName('defaultoff');
      const flagKey = uniqueFlagKey('disabled');
      const flagName = 'Default Disabled Flag';

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Create a flag
      await page.getByRole('button', { name: /create your first flag/i }).click();
      await page.getByPlaceholder('enable-new-feature').fill(flagKey);
      await page.getByPlaceholder('Enable New Feature').fill(flagName);
      await page.getByRole('button', { name: 'Create Flag' }).click();

      // Wait for flag to appear
      await expect(page.getByText(flagName)).toBeVisible({ timeout: 10000 });

      // Verify toggle is OFF (aria-checked=false)
      const isEnabled = await getFlagToggleState(page, flagKey);
      expect(isEnabled).toBe(false);
    });

    test('can create flag from empty state button', async ({ page }) => {
      const projectName = uniqueProjectName('emptystate');
      const flagKey = uniqueFlagKey('first');
      const flagName = 'First Flag';

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Click the "Create your first flag" button in empty state
      await page.getByRole('button', { name: /create your first flag/i }).click();

      // Fill modal
      await page.getByPlaceholder('enable-new-feature').fill(flagKey);
      await page.getByPlaceholder('Enable New Feature').fill(flagName);
      await page.getByRole('button', { name: 'Create Flag' }).click();

      // Verify flag was created
      await expect(page.getByText(flagName)).toBeVisible({ timeout: 10000 });
    });

    test('can cancel flag creation', async ({ page }) => {
      const projectName = uniqueProjectName('testcancel');

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Open create modal
      await page.getByRole('button', { name: /create your first flag/i }).click();

      // Fill some data
      await page.getByPlaceholder('enable-new-feature').fill('should-not-exist');
      await page.getByPlaceholder('Enable New Feature').fill('Should Not Exist');

      // Cancel using exact match
      await page.getByRole('button', { name: 'Cancel', exact: true }).click();

      // Verify modal is closed
      await expect(
        page.getByRole('heading', { name: /create flag/i })
      ).not.toBeVisible();

      // Verify empty state is still shown (flag was not created)
      await expect(page.getByText(/no feature flags yet/i)).toBeVisible();
    });

    test('flag key is sanitized to lowercase with dashes', async ({ page }) => {
      const projectName = uniqueProjectName('sanitize');
      const inputKey = 'My Special Key 123';
      const expectedKey = 'my-special-key-123';
      const flagName = 'Sanitized Flag';

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Open create modal and fill
      await page.getByRole('button', { name: /create your first flag/i }).click();
      const keyInput = page.getByPlaceholder('enable-new-feature');
      await keyInput.fill(inputKey);
      await page.getByPlaceholder('Enable New Feature').fill(flagName);

      // Check that input was sanitized
      await expect(keyInput).toHaveValue(expectedKey);

      // Submit
      await page.getByRole('button', { name: 'Create Flag' }).click();

      // Verify sanitized key is shown in table
      await expect(page.getByText(flagName)).toBeVisible({ timeout: 10000 });
      await expect(page.locator('code').filter({ hasText: expectedKey })).toBeVisible();
    });
  });

  test.describe('Toggle Flag', () => {
    test('can toggle flag from disabled to enabled', async ({ page }) => {
      const projectName = uniqueProjectName('toggle');
      const flagKey = uniqueFlagKey('toggleme');
      const flagName = 'Toggle Me';

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

      // Verify initial state is OFF
      let isEnabled = await getFlagToggleState(page, flagKey);
      expect(isEnabled).toBe(false);

      // Toggle ON
      await toggleFlag(page, flagKey);

      // Verify state is now ON
      isEnabled = await getFlagToggleState(page, flagKey);
      expect(isEnabled).toBe(true);
    });

    test('can toggle flag from enabled to disabled', async ({ page }) => {
      const projectName = uniqueProjectName('toggleoff');
      const flagKey = uniqueFlagKey('toggleoff');
      const flagName = 'Toggle Off';

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Create a flag and toggle it on
      await page.getByRole('button', { name: /create your first flag/i }).click();
      await page.getByPlaceholder('enable-new-feature').fill(flagKey);
      await page.getByPlaceholder('Enable New Feature').fill(flagName);
      await page.getByRole('button', { name: 'Create Flag' }).click();
      await expect(page.getByText(flagName)).toBeVisible({ timeout: 10000 });
      await toggleFlag(page, flagKey);

      // Verify it's ON
      let isEnabled = await getFlagToggleState(page, flagKey);
      expect(isEnabled).toBe(true);

      // Toggle OFF
      await toggleFlag(page, flagKey);

      // Verify state is now OFF
      isEnabled = await getFlagToggleState(page, flagKey);
      expect(isEnabled).toBe(false);
    });

    test('toggle persists after page refresh', async ({ page }) => {
      const projectName = uniqueProjectName('persist');
      const flagKey = uniqueFlagKey('persist');
      const flagName = 'Persist Flag';

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Create a flag and toggle it on
      await page.getByRole('button', { name: /create your first flag/i }).click();
      await page.getByPlaceholder('enable-new-feature').fill(flagKey);
      await page.getByPlaceholder('Enable New Feature').fill(flagName);
      await page.getByRole('button', { name: 'Create Flag' }).click();
      await expect(page.getByText(flagName)).toBeVisible({ timeout: 10000 });
      await toggleFlag(page, flagKey);

      // Verify it's ON
      let isEnabled = await getFlagToggleState(page, flagKey);
      expect(isEnabled).toBe(true);

      // Refresh the page
      await page.reload();

      // Wait for flags table to load
      await expect(page.getByText(flagName)).toBeVisible({ timeout: 10000 });

      // Verify state is still ON after refresh
      isEnabled = await getFlagToggleState(page, flagKey);
      expect(isEnabled).toBe(true);
    });
  });

  test.describe('Delete Flag', () => {
    test('can delete flag with confirmation', async ({ page }) => {
      const projectName = uniqueProjectName('delete');
      const flagKey = uniqueFlagKey('deleteme');
      const flagName = 'Delete Me';

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Create a flag
      await page.getByRole('button', { name: /create your first flag/i }).click();
      await page.getByPlaceholder('enable-new-feature').fill(flagKey);
      await page.getByPlaceholder('Enable New Feature').fill(flagName);
      await page.getByRole('button', { name: 'Create Flag' }).click();

      // Verify flag exists
      await expect(page.getByText(flagName)).toBeVisible({ timeout: 10000 });

      // Click delete button (trash icon)
      const row = page.locator('tr').filter({ hasText: flagKey });
      await row.locator('button').last().click();

      // Confirm delete modal appears
      await expect(
        page.getByRole('heading', { name: /delete flag/i })
      ).toBeVisible();
      await expect(
        page.getByText(/are you sure you want to delete/i)
      ).toBeVisible();

      // Confirm deletion
      await page.getByRole('button', { name: 'Delete Flag' }).click();

      // Verify flag is removed
      await expect(page.getByText(flagName)).not.toBeVisible();

      // Verify empty state is shown
      await expect(page.getByText(/no feature flags yet/i)).toBeVisible();
    });

    test('can cancel flag deletion', async ({ page }) => {
      const projectName = uniqueProjectName('keepdelete');
      const flagKey = uniqueFlagKey('keepme');
      const flagName = 'Keep Me';

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

      // Click delete button
      const row = page.locator('tr').filter({ hasText: flagKey });
      await row.locator('button').last().click();

      // Cancel using exact match
      await page.getByRole('button', { name: 'Cancel', exact: true }).click();

      // Verify modal is closed
      await expect(
        page.getByRole('heading', { name: /delete flag/i })
      ).not.toBeVisible();

      // Verify flag still exists
      await expect(page.getByText(flagName)).toBeVisible();
    });
  });

  test.describe('Flag States Per Environment', () => {
    test('flag can have different states per environment', async ({ page }) => {
      const projectName = uniqueProjectName('envstates');
      const flagKey = uniqueFlagKey('perenv');
      const flagName = 'Per Environment Flag';

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

      // We start on development tab by default - enable flag
      await selectEnvironment(page, 'Development');
      await toggleFlag(page, flagKey);

      // Verify flag is ON in development
      let isEnabled = await getFlagToggleState(page, flagKey);
      expect(isEnabled).toBe(true);

      // Switch to staging - should be OFF (independent state)
      await selectEnvironment(page, 'Staging');

      // Verify flag is OFF in staging
      isEnabled = await getFlagToggleState(page, flagKey);
      expect(isEnabled).toBe(false);

      // Switch to production - should also be OFF
      await selectEnvironment(page, 'Production');

      // Verify flag is OFF in production
      isEnabled = await getFlagToggleState(page, flagKey);
      expect(isEnabled).toBe(false);
    });

    test('environment tabs switch correctly', async ({ page }) => {
      const projectName = uniqueProjectName('envtabs');

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Go to flags page
      await goToFlagsPage(page, projectName);

      // Verify development is selected by default
      await expect(
        page.getByRole('button', { name: /development/i })
      ).toHaveClass(/border-green-600/);

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
    });

    test('toggling in one environment does not affect others', async ({
      page,
    }) => {
      const projectName = uniqueProjectName('independent');
      const flagKey = uniqueFlagKey('independent');
      const flagName = 'Independent States';

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
  });

  test.describe('Flag Detail Page', () => {
    test('can navigate to flag detail page', async ({ page }) => {
      const projectName = uniqueProjectName('detail');
      const flagKey = uniqueFlagKey('detail');
      const flagName = 'Detail Flag';

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

      // Click on flag name to go to detail page
      await page.getByText(flagName).click();

      // Verify we're on the detail page
      await expect(page).toHaveURL(new RegExp(`/flags/${flagKey}$`));
      await expect(page.getByRole('heading', { name: flagName })).toBeVisible();
      await expect(page.locator('code').filter({ hasText: flagKey }).first()).toBeVisible();
    });

    test('can toggle flag from detail page', async ({ page }) => {
      const projectName = uniqueProjectName('detailtoggle');
      const flagKey = uniqueFlagKey('detailtoggle');
      const flagName = 'Detail Toggle Flag';

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

      // Navigate to detail page
      await page.getByText(flagName).click();
      await expect(page).toHaveURL(new RegExp(`/flags/${flagKey}$`));

      // Get toggle and verify it's OFF
      const toggle = page.getByRole('switch');
      await expect(toggle).toHaveAttribute('aria-checked', 'false');

      // Toggle ON
      await toggle.click();
      await page.waitForTimeout(500);

      // Verify it's ON
      await expect(toggle).toHaveAttribute('aria-checked', 'true');

      // Go back to list and verify state persisted
      await page.getByText(/back to flags/i).click();
      await expect(page).toHaveURL(/\/projects\/[^/]+\/flags$/);

      const isEnabled = await getFlagToggleState(page, flagKey);
      expect(isEnabled).toBe(true);
    });

    test('shows SDK integration code snippets', async ({ page }) => {
      const projectName = uniqueProjectName('sdk');
      const flagKey = uniqueFlagKey('sdk');
      const flagName = 'SDK Flag';

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

      // Navigate to detail page
      await page.getByText(flagName).click();

      // Verify SDK section is visible
      await expect(
        page.getByRole('heading', { name: /sdk integration/i })
      ).toBeVisible();

      // Verify language tabs
      await expect(page.getByRole('button', { name: 'JavaScript' })).toBeVisible();
      await expect(page.getByRole('button', { name: 'Python' })).toBeVisible();
      await expect(page.getByRole('button', { name: 'Go' })).toBeVisible();
      await expect(page.getByRole('button', { name: 'Rust' })).toBeVisible();

      // Verify code snippet contains the flag key
      await expect(page.locator('pre code')).toContainText(flagKey);
    });
  });
});
