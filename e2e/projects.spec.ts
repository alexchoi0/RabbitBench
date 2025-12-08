import { test, expect } from "@playwright/test";

test.describe("Workspaces Page", () => {
  // Dev mode is enabled in playwright.config.ts, so user is authenticated

  test("displays workspaces page", async ({ page }) => {
    await page.goto("/workspaces");

    // Should show workspaces heading or create workspace prompt
    await expect(page.getByRole("heading", { name: /workspaces/i })).toBeVisible();
  });

  test("has button to create new workspace", async ({ page }) => {
    await page.goto("/workspaces");

    const newWorkspaceButton = page.getByRole("button", { name: /new workspace/i });

    await expect(newWorkspaceButton).toBeVisible();
  });

  test("new workspace page is accessible", async ({ page }) => {
    await page.goto("/workspaces/new");

    // Should show the new workspace form
    await expect(page.getByRole("heading", { name: /new workspace/i })).toBeVisible();
  });
});
