import { test, expect } from '@playwright/test';

test.describe('Workspaces', () => {
	test.describe('Create Workspace Form (unauthenticated)', () => {
		test('should redirect to login when accessing workspaces unauthenticated', async ({ page }) => {
			await page.goto('/workspaces');
			// Should redirect to login for unauthenticated users
			await expect(page).toHaveURL(/\/(login|register)/);
		});

		test('should redirect to login when accessing new workspace unauthenticated', async ({
			page
		}) => {
			await page.goto('/workspaces/new');
			// Should redirect to login for unauthenticated users
			await expect(page).toHaveURL(/\/(login|register)/);
		});
	});
});
