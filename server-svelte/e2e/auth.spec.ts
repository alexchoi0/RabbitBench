import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
	test.describe('Login Page', () => {
		test('should display login form', async ({ page }) => {
			await page.goto('/login');
			await expect(page.locator('#email')).toBeVisible();
			await expect(page.locator('#password')).toBeVisible();
			await expect(page.getByRole('button', { name: /sign in with email/i })).toBeVisible();
		});

		test('should have link to register page', async ({ page }) => {
			await page.goto('/login');
			await page.getByRole('link', { name: /sign up/i }).click();
			await expect(page).toHaveURL('/register');
		});

		test('should have GitHub OAuth button', async ({ page }) => {
			await page.goto('/login');
			await expect(page.getByRole('button', { name: /github/i })).toBeVisible();
		});

		test('should allow typing in email and password fields', async ({ page }) => {
			await page.goto('/login');
			await page.locator('#email').fill('test@example.com');
			await page.locator('#password').fill('testpassword');

			await expect(page.locator('#email')).toHaveValue('test@example.com');
			await expect(page.locator('#password')).toHaveValue('testpassword');
		});
	});

	test.describe('Register Page', () => {
		test('should display register form', async ({ page }) => {
			await page.goto('/register');
			await expect(page.locator('#name')).toBeVisible();
			await expect(page.locator('#email')).toBeVisible();
			await expect(page.locator('#password')).toBeVisible();
			await expect(page.getByRole('button', { name: /create account/i })).toBeVisible();
		});

		test('should have link to login page', async ({ page }) => {
			await page.goto('/register');
			await page.getByRole('link', { name: /sign in/i }).click();
			await expect(page).toHaveURL('/login');
		});

		test('should have GitHub OAuth button', async ({ page }) => {
			await page.goto('/register');
			await expect(page.getByRole('button', { name: /github/i })).toBeVisible();
		});

		test('should allow typing in form fields', async ({ page }) => {
			await page.goto('/register');
			await page.locator('#name').fill('Test User');
			await page.locator('#email').fill('test@example.com');
			await page.locator('#password').fill('testpassword123');

			await expect(page.locator('#name')).toHaveValue('Test User');
			await expect(page.locator('#email')).toHaveValue('test@example.com');
			await expect(page.locator('#password')).toHaveValue('testpassword123');
		});
	});
});
