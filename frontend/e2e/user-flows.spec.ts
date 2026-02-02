import { test, expect } from '@playwright/test'

test.describe('E2E Smoke - MR.DarkPromth', () => {
  test('registers and logs in a new user', async ({ page }) => {
    const uniqueId = Date.now()
    const username = `e2e_user_${uniqueId}`
    const email = `e2e_${uniqueId}@example.com`
    const password = 'TestPass123!'

    await page.goto('/register')
    await page.getByPlaceholder('Username').fill(username)
    await page.getByPlaceholder('Email address').fill(email)
    await page.locator('input[placeholder="Password"]').first().fill(password)
    await page.getByPlaceholder('Confirm password').fill(password)
    await page.getByRole('button', { name: 'Create Account' }).click()

    await expect(page.getByText('Registration successful!')).toBeVisible()
    await page.waitForURL('**/login')

    await page.getByPlaceholder('Email address').fill(email)
    await page.getByPlaceholder('Password').fill(password)
    await page.getByRole('button', { name: 'Sign In' }).click()

    await expect(page).toHaveURL('**/')
    await expect(page.getByText('HACKER DASHBOARD')).toBeVisible()
  })

  test('redirects unauthenticated users to login', async ({ page }) => {
    await page.goto('/chat')
    await expect(page).toHaveURL('**/login')
    await expect(page.getByText('Sign In')).toBeVisible()
  })
})
