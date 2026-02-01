import { test, expect } from '@playwright/test';

test.describe('E2E Tests - MR.DarkPromth', () => {
  
  // FLOW-001: New User Registration
  test('FLOW-001: New user registration and onboarding', async ({ page }) => {
    await page.goto('/login');
    await page.click('text=Create Account');
    await page.fill('[name="username"]', 'testuser');
    await page.fill('[name="email"]', 'test@example.com');
    await page.fill('[name="password"]', 'TestPass123!');
    await page.click('button[type="submit"]');
    await expect(page.locator('text=Check your email')).toBeVisible();
  });

  // FLOW-002: Free to Premium Upgrade
  test('FLOW-002: Upgrade from Free to Premium', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[name="email"]', 'free@user.com');
    await page.fill('[name="password"]', 'password');
    await page.click('button[type="submit"]');
    await page.goto('/billing');
    await expect(page.locator('text=Premium Plan')).toBeVisible();
  });

  // FLOW-003: Premium to Ultra Upgrade
  test('FLOW-003: Upgrade to Ultra tier', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[name="email"]', 'premium@user.com');
    await page.fill('[name="password"]', 'password');
    await page.click('button[type="submit"]');
    await page.goto('/billing');
    await page.click('text=Upgrade to Ultra');
    await expect(page.locator('text=Ultra terms')).toBeVisible();
  });

  // FLOW-004: GitHub OAuth
  test('FLOW-004: GitHub OAuth login', async ({ page }) => {
    await page.goto('/login');
    await page.click('text=Login with GitHub');
    await expect(page).toHaveURL(/github.com/);
  });

  // FLOW-005: Free Tier Chat
  test('FLOW-005: Chat functionality for Free tier', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[name="email"]', 'free@user.com');
    await page.fill('[name="password"]', 'password');
    await page.click('button[type="submit"]');
    await page.goto('/chat');
    await page.fill('[placeholder="Type your message..."]', 'Hello AI');
    await page.click('button[type="submit"]');
    await expect(page.locator('text=Hello AI')).toBeVisible();
  });

  // FLOW-006: Ultra Tier Chat with Jailbreak
  test('FLOW-006: Ultra tier unrestricted chat', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[name="email"]', 'ultra@user.com');
    await page.fill('[name="password"]', 'password');
    await page.click('button[type="submit"]');
    await page.goto('/chat');
    await expect(page.locator('text=Ultra Mode')).toBeVisible();
  });

  // FLOW-007: Terminal Access (Ultra Only)
  test('FLOW-007: Terminal execution for Ultra users', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[name="email"]', 'ultra@user.com');
    await page.fill('[name="password"]', 'password');
    await page.click('button[type="submit"]');
    await page.goto('/terminal');
    await page.fill('input[type="text"]', 'ls -la');
    await page.keyboard.press('Enter');
    await expect(page.locator('text=total')).toBeVisible({ timeout: 5000 });
  });

  // FLOW-008: Jailbreak Library
  test('FLOW-008: Access jailbreak prompt library', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[name="email"]', 'ultra@user.com');
    await page.fill('[name="password"]', 'password');
    await page.click('button[type="submit"]');
    await page.goto('/jailbreak');
    await expect(page.locator('text=Jailbreak Prompt Library')).toBeVisible();
  });

  // FLOW-009: Tool Execution
  test('FLOW-009: Execute tools in sandbox', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[name="email"]', 'premium@user.com');
    await page.fill('[name="password"]', 'password');
    await page.click('button[type="submit"]');
    await page.goto('/tools');
    await expect(page.locator('text=Available Tools')).toBeVisible();
  });

  // FLOW-010: Admin User Management
  test('FLOW-010: Admin user management', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[name="email"]', 'admin@system.com');
    await page.fill('[name="password"]', 'adminpass');
    await page.click('button[type="submit"]');
    await page.goto('/admin');
    await expect(page.locator('text=User Management')).toBeVisible();
  });

  // FLOW-011: Password Reset
  test('FLOW-011: Password reset flow', async ({ page }) => {
    await page.goto('/login');
    await page.click('text=Forgot Password');
    await page.fill('[name="email"]', 'user@example.com');
    await page.click('button[type="submit"]');
    await expect(page.locator('text=Reset email sent')).toBeVisible();
  });

  // FLOW-012: WebSocket Chat
  test('FLOW-012: WebSocket real-time chat', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[name="email"]', 'user@example.com');
    await page.fill('[name="password"]', 'password');
    await page.click('button[type="submit"]');
    await page.goto('/chat');
    await expect(page.locator('text=Connected')).toBeVisible();
  });

  // FLOW-013: Session Management
  test('FLOW-013: Session management and logout', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[name="email"]', 'user@example.com');
    await page.fill('[name="password"]', 'password');
    await page.click('button[type="submit"]');
    await page.click('text=Logout');
    await expect(page).toHaveURL('/login');
  });
});
