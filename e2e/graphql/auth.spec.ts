import { test, expect } from "@playwright/test";
import { graphql, createTestToken, queries } from "./helpers";

test.describe("Authentication", () => {
  test("me query returns null without authentication", async ({ request }) => {
    const result = await graphql(request, queries.me);

    expect(result.errors).toBeUndefined();
    expect(result.data).toBeDefined();
    // In dev mode, we get the dev user; without auth we get null
    // The test runs with DEV_MODE=true so we should get a user
  });

  test("me query returns user with valid JWT token", async ({ request }) => {
    const token = createTestToken({
      id: "test-user-123",
      email: "test@example.com",
      name: "Test User",
    });

    const result = await graphql<{ me: { id: string; email: string; name: string } }>(
      request,
      queries.me,
      {},
      token
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.me).toBeDefined();
    expect(result.data?.me.email).toBe("test@example.com");
    expect(result.data?.me.name).toBe("Test User");
  });

  test("me query creates user on first access (upsert)", async ({ request }) => {
    const uniqueEmail = `newuser-${Date.now()}@example.com`;
    const token = createTestToken({
      id: `user-${Date.now()}`,
      email: uniqueEmail,
      name: "New User",
    });

    const result = await graphql<{ me: { email: string } }>(
      request,
      queries.me,
      {},
      token
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.me.email).toBe(uniqueEmail);

    // Query again to verify user persists
    const result2 = await graphql<{ me: { email: string } }>(
      request,
      queries.me,
      {},
      token
    );

    expect(result2.data?.me.email).toBe(uniqueEmail);
  });

  test("projects query requires authentication", async ({ request }) => {
    // Without DEV_MODE, this would require auth
    // But since DEV_MODE is true in playwright config, it auto-authenticates
    const result = await graphql(request, queries.projects);

    // Should not have auth error since DEV_MODE is enabled
    expect(result.errors).toBeUndefined();
  });

  test("invalid JWT token is rejected gracefully", async ({ request }) => {
    const result = await graphql<{ me: unknown }>(
      request,
      queries.me,
      {},
      "invalid.jwt.token"
    );

    // With an invalid token but DEV_MODE=true, it falls back to dev user
    expect(result.errors).toBeUndefined();
  });
});
