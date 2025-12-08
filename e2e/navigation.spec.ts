import { test, expect } from "@playwright/test";

test.describe("Navigation", () => {
  test("header shows logo linking to home", async ({ page }) => {
    await page.goto("/");

    const logo = page.getByRole("link", { name: "RabbitBench" });

    await expect(logo).toBeVisible();
    await expect(logo).toHaveAttribute("href", "/");
  });

  test("footer contains expected links", async ({ page }) => {
    await page.goto("/");

    await expect(page.getByRole("link", { name: "Docs" }).last()).toBeVisible();
    await expect(page.getByRole("link", { name: "Privacy" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Terms" })).toBeVisible();
  });

  test.describe("mobile navigation", () => {
    test.beforeEach(async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
    });

    test("mobile menu button is visible on small screens", async ({ page }) => {
      await page.goto("/");

      // The hamburger menu should be visible
      const menuButton = page.getByRole("button").first();
      await expect(menuButton).toBeVisible();
    });
  });

  test.describe("desktop navigation", () => {
    test.beforeEach(async ({ page }) => {
      await page.setViewportSize({ width: 1280, height: 720 });
    });

    test("desktop nav links are visible", async ({ page }) => {
      await page.goto("/");

      await expect(page.getByRole("link", { name: "Docs" }).first()).toBeVisible();
    });
  });
});
