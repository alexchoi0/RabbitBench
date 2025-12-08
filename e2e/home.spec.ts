import { test, expect } from "@playwright/test";

test.describe("Home Page", () => {
  test.describe("when signed in (dev mode)", () => {
    // In dev mode with DEV_MODE=true, user is automatically authenticated
    test("Get Started button is visible", async ({ page }) => {
      await page.goto("/");

      const getStartedButton = page.getByRole("button", { name: "Get Started" }).first();

      await expect(getStartedButton).toBeVisible();
    });

    test("Get Started for Free button is visible", async ({ page }) => {
      await page.goto("/");

      const ctaButton = page.getByRole("button", { name: "Get Started for Free" });

      await expect(ctaButton).toBeVisible();
    });

    test("clicking Get Started navigates to workspaces page", async ({ page }) => {
      await page.goto("/");

      await page.getByRole("button", { name: "Get Started" }).first().click();

      await expect(page).toHaveURL("/workspaces");
    });
  });

  test("page displays main heading", async ({ page }) => {
    await page.goto("/");

    await expect(page.getByRole("heading", { name: /Benchmark Performance/i })).toBeVisible();
  });

  test("page displays feature cards", async ({ page }) => {
    await page.goto("/");

    await expect(page.getByRole("heading", { name: "Collect" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Visualize" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Alert" })).toBeVisible();
  });

  test("GitHub link is correct", async ({ page }) => {
    await page.goto("/");

    const githubLink = page.getByRole("link", { name: /View on GitHub/i });

    await expect(githubLink).toHaveAttribute("href", "https://github.com/alexchoi0/rabbitbench");
    await expect(githubLink).toHaveAttribute("target", "_blank");
  });

  test("navigation links are visible on desktop", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto("/");

    await expect(page.getByRole("link", { name: "Docs" }).first()).toBeVisible();
  });
});
