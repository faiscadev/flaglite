import { test, expect, Page } from '@playwright/test';

/**
 * E2E tests for FlagLite Dashboard projects CRUD.
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

// Helper to create a project via the modal
async function createProject(page: Page, projectName: string) {
  // Click New Project button
  await page.getByRole('button', { name: /new project/i }).click();

  // Fill project name in modal
  await page.getByPlaceholder(/my-awesome-app/i).fill(projectName);

  // Submit
  await page.getByRole('button', { name: /create project/i }).click();

  // Wait for navigation to the new project's flags page
  await expect(page).toHaveURL(/\/projects\/[^/]+\/flags/, { timeout: 10000 });

  // Verify we're on the flags page with the project name visible
  await expect(page.getByRole('heading', { name: /flags/i })).toBeVisible();
}

// Helper to navigate to projects list
async function goToProjectsList(page: Page) {
  await page.goto('/projects');
  await expect(page.getByRole('heading', { name: /projects/i })).toBeVisible();
}

// Helper to open project selector dropdown
async function openProjectSelector(page: Page) {
  // The project selector is a button in the sidebar with chevrons icon
  const projectSelectorButton = page.locator(
    'aside button:has(.lucide-chevrons-up-down), button:has([class*="ChevronsUpDown"])'
  );

  // Fallback: find button in sidebar that has the current project name
  const sidebarProjectButton = page.locator(
    'aside .border-b button'
  );

  const button = (await projectSelectorButton.count()) > 0
    ? projectSelectorButton
    : sidebarProjectButton;

  await button.click();

  // Wait for dropdown to appear
  await expect(page.getByText(/projects/i).first()).toBeVisible();
}

test.describe('Projects', () => {
  test.beforeEach(async ({ page }) => {
    await clearAuthState(page);
  });

  test.describe('List Projects', () => {
    test('after signup, user sees default project in list', async ({
      page,
    }) => {
      const projectName = uniqueProjectName('default');

      // Signup with a specific project name
      await signupUser(page, undefined, 'testpass123', projectName);

      // Navigate to projects list
      await goToProjectsList(page);

      // Verify the default project is visible
      await expect(
        page.getByRole('heading', { name: projectName })
      ).toBeVisible();
    });

    test('project name is displayed correctly', async ({ page }) => {
      const projectName = 'My Test Application';

      // Signup with a specific project name
      await signupUser(page, undefined, 'testpass123', projectName);

      // Navigate to projects list
      await goToProjectsList(page);

      // Verify the project name is displayed correctly (heading inside project card)
      await expect(
        page.getByRole('heading', { name: projectName })
      ).toBeVisible();

      // Verify created date is shown
      await expect(page.getByText(/created/i)).toBeVisible();
    });
  });

  test.describe('Create Project', () => {
    test('can create new project via modal', async ({ page }) => {
      const newProjectName = uniqueProjectName('new');

      // Signup first
      await signupUser(page);

      // Go to projects list
      await goToProjectsList(page);

      // Create a new project
      await createProject(page, newProjectName);

      // Navigate back to projects list
      await goToProjectsList(page);

      // Verify new project appears in list
      await expect(
        page.getByRole('heading', { name: newProjectName })
      ).toBeVisible();
    });

    test('new project has default environments', async ({ page }) => {
      const newProjectName = uniqueProjectName('envtest');

      // Signup first
      await signupUser(page);

      // Go to projects list
      await goToProjectsList(page);

      // Create a new project
      await createProject(page, newProjectName);

      // Navigate to environments page for this project
      // We're already on /projects/{id}/flags, change to /environments
      const currentUrl = page.url();
      const projectId = currentUrl.match(/\/projects\/([^/]+)\//)?.[1];
      expect(projectId).toBeTruthy();

      await page.goto(`/projects/${projectId}/environments`);

      // Wait for environments page to load
      await expect(
        page.getByRole('heading', { name: /environments/i })
      ).toBeVisible();

      // Verify default environments exist (development, staging, production)
      await expect(page.getByRole('heading', { name: /development/i })).toBeVisible();
      await expect(page.getByRole('heading', { name: /staging/i })).toBeVisible();
      await expect(page.getByRole('heading', { name: /production/i })).toBeVisible();
    });

    test('create project modal shows validation', async ({ page }) => {
      // Signup first
      await signupUser(page);

      // Go to projects list
      await goToProjectsList(page);

      // Open create modal
      await page.getByRole('button', { name: /new project/i }).click();

      // Try to submit without filling project name
      await page.getByRole('button', { name: /create project/i }).click();

      // Should show browser validation (field is required)
      const projectNameInput = page.getByPlaceholder(/my-awesome-app/i);
      const isInvalid = await projectNameInput.evaluate(
        (el: HTMLInputElement) => !el.validity.valid
      );
      expect(isInvalid).toBe(true);
    });

    test('can cancel project creation', async ({ page }) => {
      // Signup first
      await signupUser(page);

      // Go to projects list
      await goToProjectsList(page);

      // Count initial projects
      const initialProjects = await page
        .locator('.grid button')
        .count();

      // Open create modal
      await page.getByRole('button', { name: /new project/i }).click();

      // Fill project name
      await page
        .getByPlaceholder(/my-awesome-app/i)
        .fill('should-not-be-created');

      // Cancel
      await page.getByRole('button', { name: /cancel/i }).click();

      // Verify modal is closed
      await expect(
        page.getByRole('heading', { name: /create project/i })
      ).not.toBeVisible();

      // Verify project count hasn't changed
      const currentProjects = await page
        .locator('.grid button')
        .count();
      expect(currentProjects).toBe(initialProjects);
    });
  });

  test.describe('Switch Between Projects', () => {
    test('can switch between projects using project selector', async ({
      page,
    }) => {
      const project1 = uniqueProjectName('first');
      const project2 = uniqueProjectName('second');

      // Signup with first project
      await signupUser(page, undefined, 'testpass123', project1);

      // Go to projects list and create second project
      await goToProjectsList(page);
      await createProject(page, project2);

      // We're now on project2's flags page
      await expect(
        page.getByText(new RegExp(`flags for ${project2}`, 'i'))
      ).toBeVisible();

      // Open project selector
      await openProjectSelector(page);

      // Click on project1 in the dropdown
      await page.locator('[class*="absolute"]').getByRole('button', { name: project1 }).click();

      // Wait for navigation and verify we switched to project1
      await expect(page).toHaveURL(/\/projects\/[^/]+\/flags/);
      await expect(
        page.getByText(new RegExp(`flags for ${project1}`, 'i'))
      ).toBeVisible();

      // Switch back to project2
      await openProjectSelector(page);
      await page.locator('[class*="absolute"]').getByRole('button', { name: project2 }).click();

      // Verify we're back on project2
      await expect(
        page.getByText(new RegExp(`flags for ${project2}`, 'i'))
      ).toBeVisible();
    });

    test('project context changes when switching', async ({ page }) => {
      const project1 = uniqueProjectName('ctxa');
      const project2 = uniqueProjectName('ctxb');

      // Signup with first project
      await signupUser(page, undefined, 'testpass123', project1);

      // Go to projects list and create second project
      await goToProjectsList(page);
      await createProject(page, project2);

      // We're now on project2's flags page - verify by checking subtitle
      await expect(
        page.getByText(new RegExp(`flags for ${project2}`, 'i'))
      ).toBeVisible();

      // Get project2's URL to verify context
      const project2Url = page.url();
      const project2Id = project2Url.match(/\/projects\/([^/]+)\//)?.[1];
      expect(project2Id).toBeTruthy();

      // Switch to project1 via selector - wait for URL to change
      await openProjectSelector(page);
      await page.locator('[class*="absolute"]').getByRole('button', { name: project1 }).click();

      // Wait for subtitle to show project1's name (confirms context switch)
      await expect(
        page.getByText(new RegExp(`flags for ${project1}`, 'i'))
      ).toBeVisible();

      // Verify URL changed to project1
      const project1Url = page.url();
      const project1Id = project1Url.match(/\/projects\/([^/]+)\//)?.[1];
      expect(project1Id).toBeTruthy();
      expect(project1Id).not.toBe(project2Id);

      // Switch back to project2
      await openProjectSelector(page);
      await page.locator('[class*="absolute"]').getByRole('button', { name: project2 }).click();

      // Verify project2's context
      await expect(
        page.getByText(new RegExp(`flags for ${project2}`, 'i'))
      ).toBeVisible();
    });

    test('can create project from project selector dropdown', async ({
      page,
    }) => {
      const initialProject = uniqueProjectName('init');
      const newProject = uniqueProjectName('fromdd');

      // Signup with initial project
      await signupUser(page, undefined, 'testpass123', initialProject);

      // Click on the project card in the list to go to flags page
      await page.locator('.grid button').filter({ hasText: initialProject }).click();
      await expect(page).toHaveURL(/\/projects\/[^/]+\/flags/);

      // Open project selector
      await openProjectSelector(page);

      // Click "New Project" in dropdown (it's inside the dropdown, not the header button)
      await page.locator('[class*="absolute"]').getByRole('button', { name: /new project/i }).click();

      // Modal should open
      await expect(
        page.getByRole('heading', { name: /create project/i })
      ).toBeVisible();

      // Fill and submit
      await page.getByPlaceholder(/my-awesome-app/i).fill(newProject);
      await page.getByRole('button', { name: /create project/i }).click();

      // Verify navigation to new project
      await expect(page).toHaveURL(/\/projects\/[^/]+\/flags/);
      await expect(
        page.getByText(new RegExp(`flags for ${newProject}`, 'i'))
      ).toBeVisible();
    });
  });

  test.describe('Project Persistence', () => {
    test('selected project persists after page refresh', async ({ page }) => {
      const projectName = uniqueProjectName('persist');

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Click on the project card to select it
      await page.locator('.grid button').filter({ hasText: projectName }).click();
      await expect(page).toHaveURL(/\/projects\/[^/]+\/flags/);

      // Verify we're on the correct project's flags page
      await expect(
        page.getByText(new RegExp(`flags for ${projectName}`, 'i'))
      ).toBeVisible();

      // Refresh the page
      await page.reload();

      // Verify still on the same project's flags page
      await expect(page).toHaveURL(/\/projects\/[^/]+\/flags/);
      await expect(
        page.getByText(new RegExp(`flags for ${projectName}`, 'i'))
      ).toBeVisible();
    });

    test('project selection persists in localStorage', async ({ page }) => {
      const projectName = uniqueProjectName('storage');

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Click on the project card to select it
      await page.locator('.grid button').filter({ hasText: projectName }).click();
      await expect(page).toHaveURL(/\/projects\/[^/]+\/flags/);

      // Check localStorage for project selection
      const storedProjectId = await page.evaluate(() =>
        localStorage.getItem('flaglite_selected_project')
      );
      expect(storedProjectId).not.toBeNull();

      // Refresh and verify the selection persists
      await page.reload();

      // Should still have the project in localStorage
      const storedProjectIdAfter = await page.evaluate(() =>
        localStorage.getItem('flaglite_selected_project')
      );
      expect(storedProjectIdAfter).toBe(storedProjectId);
    });

    test('navigating directly to project URL loads correct project', async ({
      page,
    }) => {
      const projectName = uniqueProjectName('directnav');

      // Signup with a project
      await signupUser(page, undefined, 'testpass123', projectName);

      // Click on the project card to go to flags page
      await page.locator('.grid button').filter({ hasText: projectName }).click();
      await expect(page).toHaveURL(/\/projects\/[^/]+\/flags/);

      // Get the project ID from URL
      const projectUrl = page.url();
      const projectId = projectUrl.match(/\/projects\/([^/]+)\//)?.[1];
      expect(projectId).toBeTruthy();

      // Navigate away
      await page.goto('/projects');
      await expect(page.getByRole('heading', { name: /projects/i })).toBeVisible();

      // Navigate directly to the project
      await page.goto(`/projects/${projectId}/flags`);

      // Verify the correct project is loaded
      await expect(
        page.getByText(new RegExp(`flags for ${projectName}`, 'i'))
      ).toBeVisible();
    });
  });
});
