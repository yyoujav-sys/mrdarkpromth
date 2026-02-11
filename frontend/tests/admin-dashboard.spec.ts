import { test, expect } from '@playwright/test';

test.describe('Admin Dashboard', () => {
    test.beforeEach(async ({ page }) => {
        // Mock auth token and user data for testing
        await page.addInitScript(() => {
            // Set token
            localStorage.setItem('token', 'test-admin-token');
            // Mock Zustand auth store state
            const authState = {
                state: {
                    isAuthenticated: true,
                    user: {
                        id: 'test-user',
                        email: 'admin@test.com',
                        username: 'testadmin',
                        tier: 'premium'
                    },
                    token: 'test-admin-token'
                },
                version: 0
            };
            localStorage.setItem('auth-storage', JSON.stringify(authState));
        });
    });

    test('should load dashboard page', async ({ page }) => {
        await page.goto('/app/admin/dashboard');

        // Wait for page to load and check for admin dashboard elements
        await page.waitForLoadState('networkidle');

        // Check for Admin Dashboard heading specifically
        await expect(page.getByRole('heading', { name: 'Admin Dashboard' })).toBeVisible({ timeout: 10000 });
    });

    test('should display metrics cards', async ({ page }) => {
        // Mock API response
        await page.route('**/api/admin/dashboard/summary', async (route) => {
            await route.fulfill({
                status: 200,
                contentType: 'application/json',
                body: JSON.stringify({
                    agent_id: 'test-agent',
                    status: 'active',
                    system: {
                        cpu_usage: 45.5,
                        memory_usage: 2048
                    },
                    requests: {
                        total: 1000,
                        successful: 950,
                        failed: 50,
                        avg_latency_ms: 120.5,
                        requests_per_second: 10.5,
                        error_rate: 5.0
                    },
                    learning: {
                        total_corrections: 100,
                        successful_corrections: 85,
                        failed_corrections: 15,
                        average_confidence: 0.87,
                        average_validation_score: 0.92,
                        most_common_errors: ['timeout', 'invalid_input']
                    },
                    timestamp: new Date().toISOString()
                })
            });
        });

        await page.route('**/api/admin/users', async (route) => {
            await route.fulfill({
                status: 200,
                contentType: 'application/json',
                body: JSON.stringify({ users: [] })
            });
        });

        await page.goto('/app/admin/dashboard');
        await page.waitForLoadState('networkidle');

        // Wait for metrics to load
        await expect(page.getByText('CPU Usage')).toBeVisible();
        await expect(page.getByText('45.5%')).toBeVisible();

        await expect(page.getByText('Requests / Sec')).toBeVisible();
        await expect(page.getByText('10.50')).toBeVisible();

        await expect(page.getByText('Avg Latency')).toBeVisible();
        await expect(page.getByText('121 ms')).toBeVisible();

        await expect(page.getByText('Error Rate')).toBeVisible();
        await expect(page.getByText('5.00%')).toBeVisible();

        await expect(page.getByText('Self-Corrections')).toBeVisible();
        await expect(page.getByText('100')).toBeVisible();

        await expect(page.getByText('Fix Success Rate')).toBeVisible();
        await expect(page.getByText('85.0%')).toBeVisible();
    });

    test('should refresh metrics automatically', async ({ page }) => {
        let requestCount = 0;

        await page.route('**/api/admin/dashboard/summary', async (route) => {
            requestCount++;
            await route.fulfill({
                status: 200,
                contentType: 'application/json',
                body: JSON.stringify({
                    agent_id: 'test-agent',
                    status: 'active',
                    system: { cpu_usage: 50 + requestCount, memory_usage: 2048 },
                    requests: { total: 1000, successful: 950, failed: 50, avg_latency_ms: 100, requests_per_second: 10, error_rate: 5 },
                    learning: { total_corrections: 100, successful_corrections: 85, failed_corrections: 15, average_confidence: 0.87, average_validation_score: 0.92, most_common_errors: [] },
                    timestamp: new Date().toISOString()
                })
            });
        });

        await page.route('**/api/admin/users', async (route) => {
            await route.fulfill({
                status: 200,
                contentType: 'application/json',
                body: JSON.stringify({ users: [] })
            });
        });

        await page.goto('/app/admin/dashboard');
        await page.waitForLoadState('networkidle');

        // Wait for initial load
        await expect(page.getByText('CPU Usage')).toBeVisible();

        // Wait for auto-refresh (5 seconds interval + buffer)
        await page.waitForTimeout(6000);

        // Should have made at least 2 requests (initial + 1 refresh)
        expect(requestCount).toBeGreaterThanOrEqual(2);
    });
});
