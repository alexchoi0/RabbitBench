import { test, expect } from '@playwright/test';

test.describe('Home Page', () => {
	test('should display the home page', async ({ page }) => {
		await page.goto('/');
		await expect(page).toHaveTitle(/Driftwatch/);
	});

	test('should have navigation elements', async ({ page }) => {
		await page.goto('/');
		await expect(page.getByRole('link', { name: 'Driftwatch' })).toBeVisible();
		await expect(page.getByRole('navigation').getByRole('link', { name: 'Docs' })).toBeVisible();
		await expect(page.getByRole('button', { name: /sign in/i })).toBeVisible();
	});

	test('should navigate to login page via Sign In', async ({ page }) => {
		await page.goto('/');
		await page.getByRole('button', { name: /sign in/i }).click();
		await expect(page).toHaveURL('/login');
	});

	test('should navigate to register page via Get Started', async ({ page }) => {
		await page.goto('/');
		await page.getByRole('button', { name: /get started/i }).first().click();
		await expect(page).toHaveURL('/register');
	});

	test('should display hero section', async ({ page }) => {
		await page.goto('/');
		await expect(page.getByRole('heading', { name: /guard your performance/i })).toBeVisible();
	});

	test('should display features section', async ({ page }) => {
		await page.goto('/');
		await expect(page.getByRole('heading', { name: /everything you need/i })).toBeVisible();
	});
});
