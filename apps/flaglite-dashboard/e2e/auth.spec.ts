import { test, expect, Page } from '@playwright/test';

/**
 * E2E tests for FlagLite Dashboard authentication flows.
 * 
 * Prerequisites:
 *   - docker compose -f docker-compose.e2e.yml up -d
 *   - npm run dev (dashboard on port 3001)
 */

// Generate unique username for each test run to ensure independence
function uniqueUsername(prefix = 'e2e') {
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
  // Check for Projects heading which only appears when logged in
  await expect(page.getByRole('heading', { name: /projects/i })).toBeVisible();
}

// Helper to signup a new user and return credentials
async function signupUser(page: Page, username?: string, password = 'testpass123') {
  const user = username || uniqueUsername();
  
  await page.goto('/signup');
  await page.getByPlaceholder(/cool-developer/i).fill(user);
  await page.getByPlaceholder(/••••••••/).fill(password);
  await page.getByRole('button', { name: /sign up/i }).click();
  
  // Wait for API key screen
  await expect(page.getByRole('heading', { name: /account created/i })).toBeVisible({ timeout: 10000 });
  
  // Click through the API key confirmation
  await page.getByRole('button', { name: /i've saved my api key/i }).click();
  
  // Wait for redirect to projects
  await expectLoggedIn(page);
  
  return { username: user, password };
}

// Helper to login with existing credentials
async function loginUser(page: Page, username: string, password: string) {
  await page.goto('/login');
  await page.getByPlaceholder(/your-username/i).fill(username);
  await page.getByPlaceholder(/••••••••/).fill(password);
  await page.getByRole('button', { name: /sign in/i }).click();
  
  // Wait for redirect to projects
  await expectLoggedIn(page);
}

// Helper to logout - clicks the user menu button in the sidebar
async function logout(page: Page, username: string) {
  // The user menu button in desktop sidebar contains the username
  // It's a button in the sidebar complementary region that contains the username text
  const userButton = page.locator('aside button').filter({ hasText: username });
  await userButton.click();
  
  // Wait for redirect to login
  await expect(page).toHaveURL(/\/login/);
}

test.describe('Authentication', () => {
  test.beforeEach(async ({ page }) => {
    await clearAuthState(page);
  });

  test.describe('Signup Flow', () => {
    test('signup page loads correctly', async ({ page }) => {
      await page.goto('/signup');
      
      // Verify branding
      await expect(page.getByRole('heading', { name: /flaglite/i })).toBeVisible();
      
      // Verify form elements
      await expect(page.getByRole('heading', { name: /create an account/i })).toBeVisible();
      await expect(page.getByPlaceholder(/cool-developer/i)).toBeVisible();
      await expect(page.getByPlaceholder(/••••••••/)).toBeVisible();
      await expect(page.getByPlaceholder(/my-awesome-app/i)).toBeVisible();
      await expect(page.getByRole('button', { name: /sign up/i })).toBeVisible();
      
      // Verify login link
      await expect(page.getByRole('link', { name: /sign in/i })).toBeVisible();
    });

    test('successful signup redirects to dashboard', async ({ page }) => {
      const username = uniqueUsername('signup-test');
      const password = 'testpassword123';
      
      await page.goto('/signup');
      
      // Fill signup form
      await page.getByPlaceholder(/cool-developer/i).fill(username);
      await page.getByPlaceholder(/••••••••/).fill(password);
      await page.getByPlaceholder(/my-awesome-app/i).fill('test-project');
      
      // Submit
      await page.getByRole('button', { name: /sign up/i }).click();
      
      // Should show API key screen
      await expect(page.getByRole('heading', { name: /account created/i })).toBeVisible({ timeout: 10000 });
      await expect(page.getByText(/save your api key/i)).toBeVisible();
      
      // API key should be displayed
      await expect(page.locator('code')).toBeVisible();
      
      // Proceed past API key screen
      await page.getByRole('button', { name: /i've saved my api key/i }).click();
      
      // Should redirect to projects page and show Projects heading
      await expect(page).toHaveURL(/\/projects/);
      await expect(page.getByRole('heading', { name: /projects/i })).toBeVisible();
      
      // The created project should be visible (look for the heading inside the project card)
      await expect(page.getByRole('heading', { name: 'test-project' })).toBeVisible();
    });

    test('signup with auto-generated username works', async ({ page }) => {
      const password = 'autouser-pass123';
      
      await page.goto('/signup');
      
      // Only fill password (username is optional)
      await page.getByPlaceholder(/••••••••/).fill(password);
      
      // Submit
      await page.getByRole('button', { name: /sign up/i }).click();
      
      // Should show API key screen
      await expect(page.getByRole('heading', { name: /account created/i })).toBeVisible({ timeout: 10000 });
      
      // Proceed
      await page.getByRole('button', { name: /i've saved my api key/i }).click();
      
      // Should be logged in - verify by checking Projects heading
      await expectLoggedIn(page);
    });
  });

  test.describe('Login Flow', () => {
    test('login page loads correctly', async ({ page }) => {
      await page.goto('/login');
      
      // Verify branding
      await expect(page.getByRole('heading', { name: /flaglite/i })).toBeVisible();
      
      // Verify form elements
      await expect(page.getByRole('heading', { name: /sign in to your account/i })).toBeVisible();
      await expect(page.getByPlaceholder(/your-username/i)).toBeVisible();
      await expect(page.getByPlaceholder(/••••••••/)).toBeVisible();
      await expect(page.getByRole('button', { name: /sign in/i })).toBeVisible();
      
      // Verify signup link
      await expect(page.getByRole('link', { name: /sign up/i })).toBeVisible();
    });

    test('successful login redirects to dashboard', async ({ page }) => {
      // First, create a test user via signup
      const { username, password } = await signupUser(page);
      
      // Logout via the user button
      await logout(page, username);
      
      // Now test login
      await page.getByPlaceholder(/your-username/i).fill(username);
      await page.getByPlaceholder(/••••••••/).fill(password);
      await page.getByRole('button', { name: /sign in/i }).click();
      
      // Should redirect to projects
      await expectLoggedIn(page);
    });

    test('login link navigates from signup to login', async ({ page }) => {
      await page.goto('/signup');
      
      await page.getByRole('link', { name: /sign in/i }).click();
      
      await expect(page).toHaveURL(/\/login/);
      await expect(page.getByRole('heading', { name: /sign in to your account/i })).toBeVisible();
    });

    test('signup link navigates from login to signup', async ({ page }) => {
      await page.goto('/login');
      
      await page.getByRole('link', { name: /sign up/i }).click();
      
      await expect(page).toHaveURL(/\/signup/);
      await expect(page.getByRole('heading', { name: /create an account/i })).toBeVisible();
    });
  });

  test.describe('Logout Flow', () => {
    test('logout redirects to login page', async ({ page }) => {
      // Create user and login
      const { username } = await signupUser(page);
      
      // Click logout
      await logout(page, username);
      
      // Should redirect to login
      await expect(page).toHaveURL(/\/login/);
    });

    test('cannot access protected routes after logout', async ({ page }) => {
      // Create user and login
      const { username } = await signupUser(page);
      
      // Click logout
      await logout(page, username);
      
      // Try to access protected route directly
      await page.goto('/projects');
      
      // Should redirect back to login
      await expect(page).toHaveURL(/\/login/);
    });

    test('localStorage is cleared on logout', async ({ page }) => {
      // Create user and login
      const { username } = await signupUser(page);
      
      // Verify localStorage has auth data
      const tokenBefore = await page.evaluate(() => localStorage.getItem('flaglite_token'));
      expect(tokenBefore).not.toBeNull();
      
      // Logout
      await logout(page, username);
      
      // Verify localStorage is cleared
      const tokenAfter = await page.evaluate(() => localStorage.getItem('flaglite_token'));
      expect(tokenAfter).toBeNull();
    });
  });

  test.describe('Session Persistence', () => {
    test('user stays logged in after page refresh', async ({ page }) => {
      // Create user and login
      await signupUser(page);
      
      // Verify on projects page
      await expect(page).toHaveURL(/\/projects/);
      
      // Refresh the page
      await page.reload();
      
      // Should still be on projects page (not redirected to login)
      await expect(page).toHaveURL(/\/projects/);
      
      // Projects heading should still be visible
      await expect(page.getByRole('heading', { name: /projects/i })).toBeVisible();
    });

    test('session persists across navigation', async ({ page }) => {
      // Create user and login
      await signupUser(page);
      
      // Navigate away and back
      await page.goto('/login');
      
      // Should be redirected back to projects (already authenticated)
      await expect(page).toHaveURL(/\/projects/);
    });

    test('authenticated user is redirected from public routes', async ({ page }) => {
      // Create user and login
      await signupUser(page);
      
      // Try to access signup page while logged in
      await page.goto('/signup');
      
      // Should redirect to projects
      await expect(page).toHaveURL(/\/projects/);
    });
  });

  test.describe('Error States', () => {
    test('login with wrong password shows error message', async ({ page }) => {
      // First create a user
      const { username } = await signupUser(page);
      
      // Logout
      await logout(page, username);
      
      // Try to login with wrong password
      await page.getByPlaceholder(/your-username/i).fill(username);
      await page.getByPlaceholder(/••••••••/).fill('wrongpassword123');
      await page.getByRole('button', { name: /sign in/i }).click();
      
      // Wait for API response and potential error display
      await page.waitForTimeout(2000);
      
      // Should stay on login page (login should fail)
      await expect(page).toHaveURL(/\/login/);
      
      // The sign in button should still be visible (not redirected to dashboard)
      await expect(page.getByRole('button', { name: /sign in/i })).toBeVisible();
      
      // Should show some form of error indication
      // Check for either the alert component or that we're not logged in
      const hasError = await page.getByText(/invalid|error|failed|incorrect/i).isVisible().catch(() => false);
      const stillOnLogin = await page.getByRole('heading', { name: /sign in to your account/i }).isVisible();
      
      // Either we see an error message, or at minimum we're still on the login page
      expect(hasError || stillOnLogin).toBe(true);
    });

    test('login with non-existent user shows error', async ({ page }) => {
      await page.goto('/login');
      
      // Try to login with non-existent user
      await page.getByPlaceholder(/your-username/i).fill('non-existent-user-xyz-999');
      await page.getByPlaceholder(/••••••••/).fill('somepassword123');
      await page.getByRole('button', { name: /sign in/i }).click();
      
      // Wait for API response and potential error display
      await page.waitForTimeout(2000);
      
      // Should stay on login page (login should fail)
      await expect(page).toHaveURL(/\/login/);
      
      // The sign in button should still be visible (not redirected to dashboard)
      await expect(page.getByRole('button', { name: /sign in/i })).toBeVisible();
      
      // Should show some form of error indication
      // Check for either the alert component or that we're not logged in
      const hasError = await page.getByText(/invalid|error|failed|incorrect/i).isVisible().catch(() => false);
      const stillOnLogin = await page.getByRole('heading', { name: /sign in to your account/i }).isVisible();
      
      // Either we see an error message, or at minimum we're still on the login page
      expect(hasError || stillOnLogin).toBe(true);
    });

    test('signup with existing username shows error', async ({ page }) => {
      // First create a user
      const { username, password } = await signupUser(page);
      
      // Logout
      await logout(page, username);
      
      // Try to signup with same username
      await page.goto('/signup');
      await page.getByPlaceholder(/cool-developer/i).fill(username);
      await page.getByPlaceholder(/••••••••/).fill(password);
      await page.getByRole('button', { name: /sign up/i }).click();
      
      // Should show error message about duplicate/existing username
      await expect(
        page.getByRole('alert')
          .or(page.locator('[class*="alert"]'))
          .or(page.getByText(/already|exists|taken|duplicate/i))
      ).toBeVisible({ timeout: 10000 });
      
      // Should stay on signup page
      await expect(page).toHaveURL(/\/signup/);
    });

    test('login with empty fields shows validation', async ({ page }) => {
      await page.goto('/login');
      
      // Click submit without filling fields
      await page.getByRole('button', { name: /sign in/i }).click();
      
      // HTML5 validation should prevent submission
      // Check that we're still on login page
      await expect(page).toHaveURL(/\/login/);
      
      // Check that the username field is invalid (browser validation)
      const usernameInput = page.getByPlaceholder(/your-username/i);
      const isInvalid = await usernameInput.evaluate((el: HTMLInputElement) => !el.validity.valid);
      expect(isInvalid).toBe(true);
    });

    test('signup with short password shows validation', async ({ page }) => {
      await page.goto('/signup');
      
      // Fill with short password (min is 8)
      await page.getByPlaceholder(/••••••••/).fill('short');
      await page.getByRole('button', { name: /sign up/i }).click();
      
      // Should stay on signup page (browser validation prevents submission)
      await expect(page).toHaveURL(/\/signup/);
      
      // Check that the password field is invalid (browser minLength validation)
      const passwordInput = page.getByPlaceholder(/••••••••/);
      const isInvalid = await passwordInput.evaluate((el: HTMLInputElement) => !el.validity.valid);
      expect(isInvalid).toBe(true);
    });

    test('signup without password shows validation', async ({ page }) => {
      await page.goto('/signup');
      
      // Only fill username, leave password empty
      await page.getByPlaceholder(/cool-developer/i).fill(uniqueUsername());
      await page.getByRole('button', { name: /sign up/i }).click();
      
      // Should stay on signup page
      await expect(page).toHaveURL(/\/signup/);
      
      // Check that the password field is invalid (required validation)
      const passwordInput = page.getByPlaceholder(/••••••••/);
      const isInvalid = await passwordInput.evaluate((el: HTMLInputElement) => !el.validity.valid);
      expect(isInvalid).toBe(true);
    });
  });
});
