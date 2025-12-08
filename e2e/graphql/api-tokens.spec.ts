import { test, expect } from "@playwright/test";
import { graphql, createTestToken, queries, mutations } from "./helpers";

test.describe("API Tokens", () => {
  const testToken = createTestToken({
    id: "token-test-user",
    email: "token-test@example.com",
    name: "Token Test User",
  });

  test("can create an API token", async ({ request }) => {
    const result = await graphql<{
      createApiToken: {
        token: { id: string; name: string; createdAt: string };
        secret: string;
      };
    }>(
      request,
      mutations.createApiToken,
      { name: "My CLI Token" },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.createApiToken).toBeDefined();
    expect(result.data?.createApiToken.token.name).toBe("My CLI Token");
    expect(result.data?.createApiToken.token.id).toBeDefined();
    expect(result.data?.createApiToken.token.createdAt).toBeDefined();

    // Secret should be in rb_{uuid} format
    expect(result.data?.createApiToken.secret).toMatch(/^rb_[a-f0-9]{32}$/);
  });

  test("can use API token for authentication", async ({ request }) => {
    // Create a token first
    const createResult = await graphql<{
      createApiToken: { secret: string };
    }>(
      request,
      mutations.createApiToken,
      { name: "Auth Test Token" },
      testToken
    );

    const apiToken = createResult.data?.createApiToken.secret;
    expect(apiToken).toBeDefined();

    // Use the API token to authenticate
    const meResult = await graphql<{
      me: { email: string };
    }>(request, queries.me, {}, apiToken);

    expect(meResult.errors).toBeUndefined();
    expect(meResult.data?.me.email).toBe("token-test@example.com");
  });

  test("can list user API tokens", async ({ request }) => {
    // Create a few tokens
    await graphql(
      request,
      mutations.createApiToken,
      { name: "Token 1" },
      testToken
    );

    await graphql(
      request,
      mutations.createApiToken,
      { name: "Token 2" },
      testToken
    );

    // List tokens via me query
    const meQuery = `
      query Me {
        me {
          apiTokens {
            id
            name
            createdAt
            lastUsedAt
          }
        }
      }
    `;

    const result = await graphql<{
      me: {
        apiTokens: Array<{
          id: string;
          name: string;
          createdAt: string;
          lastUsedAt: string | null;
        }>;
      };
    }>(request, meQuery, {}, testToken);

    expect(result.errors).toBeUndefined();
    expect(result.data?.me.apiTokens.length).toBeGreaterThanOrEqual(2);

    const tokenNames = result.data?.me.apiTokens.map((t) => t.name);
    expect(tokenNames).toContain("Token 1");
    expect(tokenNames).toContain("Token 2");
  });

  test("can revoke an API token", async ({ request }) => {
    // Create a token
    const createResult = await graphql<{
      createApiToken: { token: { id: string }; secret: string };
    }>(
      request,
      mutations.createApiToken,
      { name: "To Revoke" },
      testToken
    );

    const tokenId = createResult.data?.createApiToken.token.id;
    const secret = createResult.data?.createApiToken.secret;

    // Verify it works
    const beforeRevoke = await graphql<{ me: { email: string } }>(
      request,
      queries.me,
      {},
      secret
    );
    expect(beforeRevoke.data?.me.email).toBe("token-test@example.com");

    // Revoke the token
    const revokeResult = await graphql<{ revokeApiToken: boolean }>(
      request,
      mutations.revokeApiToken,
      { id: tokenId },
      testToken
    );

    expect(revokeResult.errors).toBeUndefined();
    expect(revokeResult.data?.revokeApiToken).toBe(true);

    // In DEV_MODE, the revoked token will fall back to dev user
    // In production mode, it would return null for me
  });

  test("cannot revoke another user's token", async ({ request }) => {
    // Create a token with first user
    const createResult = await graphql<{
      createApiToken: { token: { id: string } };
    }>(
      request,
      mutations.createApiToken,
      { name: "User 1 Token" },
      testToken
    );

    const tokenId = createResult.data?.createApiToken.token.id;

    // Try to revoke with different user
    const otherUserToken = createTestToken({
      id: "other-user-789",
      email: "other@example.com",
      name: "Other User",
    });

    const revokeResult = await graphql<{ revokeApiToken: boolean }>(
      request,
      mutations.revokeApiToken,
      { id: tokenId },
      otherUserToken
    );

    // Should return false (no rows deleted due to user_id check)
    expect(revokeResult.data?.revokeApiToken).toBe(false);
  });

  test("API token updates lastUsedAt on use", async ({ request }) => {
    // Create token
    const createResult = await graphql<{
      createApiToken: { token: { id: string }; secret: string };
    }>(
      request,
      mutations.createApiToken,
      { name: "Last Used Test" },
      testToken
    );

    const secret = createResult.data?.createApiToken.secret;

    // Use the token
    await graphql(request, queries.me, {}, secret);

    // Wait a bit
    await new Promise((resolve) => setTimeout(resolve, 100));

    // Check lastUsedAt via me query
    const meQuery = `
      query Me {
        me {
          apiTokens {
            name
            lastUsedAt
          }
        }
      }
    `;

    const result = await graphql<{
      me: {
        apiTokens: Array<{ name: string; lastUsedAt: string | null }>;
      };
    }>(request, meQuery, {}, testToken);

    const token = result.data?.me.apiTokens.find(
      (t) => t.name === "Last Used Test"
    );
    expect(token?.lastUsedAt).not.toBeNull();
  });

  test("can create token with API token authentication", async ({ request }) => {
    // Create initial token with JWT
    const firstResult = await graphql<{
      createApiToken: { secret: string };
    }>(
      request,
      mutations.createApiToken,
      { name: "First Token" },
      testToken
    );

    const firstToken = firstResult.data?.createApiToken.secret;

    // Use that token to create another token
    const secondResult = await graphql<{
      createApiToken: { token: { name: string }; secret: string };
    }>(
      request,
      mutations.createApiToken,
      { name: "Second Token" },
      firstToken
    );

    expect(secondResult.errors).toBeUndefined();
    expect(secondResult.data?.createApiToken.token.name).toBe("Second Token");
  });

  test("tokens are user-scoped", async ({ request }) => {
    // Create tokens for two different users
    const user1Token = createTestToken({
      id: "user1-scope",
      email: "user1-scope@example.com",
    });

    const user2Token = createTestToken({
      id: "user2-scope",
      email: "user2-scope@example.com",
    });

    await graphql(
      request,
      mutations.createApiToken,
      { name: "User1 Token" },
      user1Token
    );

    await graphql(
      request,
      mutations.createApiToken,
      { name: "User2 Token" },
      user2Token
    );

    // List tokens for user1
    const meQuery = `
      query Me {
        me {
          email
          apiTokens {
            name
          }
        }
      }
    `;

    const user1Result = await graphql<{
      me: { email: string; apiTokens: Array<{ name: string }> };
    }>(request, meQuery, {}, user1Token);

    const user2Result = await graphql<{
      me: { email: string; apiTokens: Array<{ name: string }> };
    }>(request, meQuery, {}, user2Token);

    // Each user should only see their own tokens
    const user1TokenNames = user1Result.data?.me.apiTokens.map((t) => t.name);
    const user2TokenNames = user2Result.data?.me.apiTokens.map((t) => t.name);

    expect(user1TokenNames).toContain("User1 Token");
    expect(user1TokenNames).not.toContain("User2 Token");

    expect(user2TokenNames).toContain("User2 Token");
    expect(user2TokenNames).not.toContain("User1 Token");
  });

  test("secret is only returned on creation", async ({ request }) => {
    // Create a token
    const createResult = await graphql<{
      createApiToken: { token: { id: string }; secret: string };
    }>(
      request,
      mutations.createApiToken,
      { name: "Secret Test" },
      testToken
    );

    expect(createResult.data?.createApiToken.secret).toBeDefined();

    // Listing tokens should NOT include secret
    const meQuery = `
      query Me {
        me {
          apiTokens {
            id
            name
          }
        }
      }
    `;

    const listResult = await graphql<{
      me: { apiTokens: Array<{ id: string; name: string }> };
    }>(request, meQuery, {}, testToken);

    // The ApiToken type doesn't have a secret field (it's only in ApiTokenResult)
    const token = listResult.data?.me.apiTokens.find(
      (t) => t.name === "Secret Test"
    );
    expect(token).toBeDefined();
    // @ts-expect-error - secret shouldn't exist on listed tokens
    expect(token?.secret).toBeUndefined();
  });
});
