import { test, expect } from "@playwright/test";

test.describe("Workspaces Page", () => {
  test("displays workspaces heading", async ({ page }) => {
    await page.goto("/workspaces");

    await expect(page.getByRole("heading", { name: "Workspaces" })).toBeVisible();
  });

  test("displays page description", async ({ page }) => {
    await page.goto("/workspaces");

    await expect(page.getByText("Manage your Cargo workspaces for benchmark tracking")).toBeVisible();
  });

  test("has new workspace button", async ({ page }) => {
    await page.goto("/workspaces");

    await expect(page.getByRole("button", { name: "New Workspace" })).toBeVisible();
  });

  test("shows workspaces list or empty state", async ({ page }) => {
    await page.goto("/workspaces");

    const emptyState = page.getByText("No workspaces yet");
    const workspacesList = page.locator("[class*='grid']").filter({ has: page.locator("a[href^='/workspaces/']") });

    const hasEmptyState = await emptyState.isVisible().catch(() => false);
    const hasWorkspaces = await workspacesList.isVisible().catch(() => false);

    expect(hasEmptyState || hasWorkspaces).toBe(true);
  });

  test("new workspace button navigates to create page", async ({ page }) => {
    await page.goto("/workspaces");

    await page.getByRole("button", { name: "New Workspace" }).click();
    await expect(page).toHaveURL("/workspaces/new");
  });
});
