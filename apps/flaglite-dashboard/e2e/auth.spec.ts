import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test('login page loads correctly', async ({ page }) => {
    // Navigate to login page
    await page.goto('/login');
    
    // Verify FlagLite branding is present
    await expect(page.getByRole('heading', { name: /flaglite/i })).toBeVisible();
    
    // Verify sign in form elements
    await expect(page.getByRole('heading', { name: /sign in to your account/i })).toBeVisible();
    await expect(page.getByPlaceholder(/your-username/i)).toBeVisible();
    await expect(page.getByPlaceholder(/••••••••/)).toBeVisible();
    await expect(page.getByRole('button', { name: /sign in/i })).toBeVisible();
    
    // Verify signup link
    await expect(page.getByRole('link', { name: /sign up/i })).toBeVisible();
  });

  test('signup link navigates to signup page', async ({ page }) => {
    await page.goto('/login');
    
    // Click signup link
    await page.getByRole('link', { name: /sign up/i }).click();
    
    // Verify we're on signup page
    await expect(page).toHaveURL(/\/signup/);
  });

  test('login form shows validation for empty fields', async ({ page }) => {
    await page.goto('/login');
    
    // Try to submit empty form
    await page.getByRole('button', { name: /sign in/i }).click();
    
    // Browser validation should prevent submission (required fields)
    // The form should still be on login page
    await expect(page).toHaveURL(/\/login/);
  });
});
