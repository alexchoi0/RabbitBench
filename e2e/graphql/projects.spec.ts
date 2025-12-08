import { test, expect } from "@playwright/test";
import { graphql, createTestToken, uniqueSlug, queries, mutations } from "./helpers";

test.describe("Projects CRUD", () => {
  const testToken = createTestToken({
    id: "project-test-user",
    email: "project-test@example.com",
    name: "Project Test User",
  });

  test("can create a new project", async ({ request }) => {
    const slug = uniqueSlug("create");

    const result = await graphql<{
      createProject: { id: string; slug: string; name: string; description: string; public: boolean };
    }>(
      request,
      mutations.createProject,
      {
        input: {
          slug,
          name: "Test Project",
          description: "A test project",
          public: false,
        },
      },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.createProject).toBeDefined();
    expect(result.data?.createProject.slug).toBe(slug);
    expect(result.data?.createProject.name).toBe("Test Project");
    expect(result.data?.createProject.description).toBe("A test project");
    expect(result.data?.createProject.public).toBe(false);
  });

  test("can create a public project", async ({ request }) => {
    const slug = uniqueSlug("public");

    const result = await graphql<{
      createProject: { public: boolean };
    }>(
      request,
      mutations.createProject,
      {
        input: {
          slug,
          name: "Public Project",
          public: true,
        },
      },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.createProject.public).toBe(true);
  });

  test("cannot create project with duplicate slug", async ({ request }) => {
    const slug = uniqueSlug("duplicate");

    // Create first project
    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "First" } },
      testToken
    );

    // Try to create second project with same slug
    const result = await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Second" } },
      testToken
    );

    expect(result.errors).toBeDefined();
    expect(result.errors?.[0].message).toContain("Unique constraint");
  });

  test("can list user projects", async ({ request }) => {
    const slug = uniqueSlug("list");

    // Create a project first
    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "List Test" } },
      testToken
    );

    const result = await graphql<{
      projects: Array<{ slug: string; name: string }>;
    }>(request, queries.projects, {}, testToken);

    expect(result.errors).toBeUndefined();
    expect(result.data?.projects).toBeDefined();
    expect(Array.isArray(result.data?.projects)).toBe(true);

    const found = result.data?.projects.find((p) => p.slug === slug);
    expect(found).toBeDefined();
  });

  test("can get a single project by slug", async ({ request }) => {
    const slug = uniqueSlug("single");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Single Project", description: "Test" } },
      testToken
    );

    const result = await graphql<{
      project: {
        slug: string;
        name: string;
        description: string;
        branches: unknown[];
        testbeds: unknown[];
        measures: unknown[];
        benchmarks: unknown[];
        thresholds: unknown[];
      };
    }>(request, queries.project, { slug }, testToken);

    expect(result.errors).toBeUndefined();
    expect(result.data?.project).toBeDefined();
    expect(result.data?.project.slug).toBe(slug);
    expect(result.data?.project.name).toBe("Single Project");
    // Should have default latency measure
    expect(result.data?.project.measures.length).toBeGreaterThanOrEqual(1);
  });

  test("can update a project", async ({ request }) => {
    const slug = uniqueSlug("update");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Original Name", description: "Original" } },
      testToken
    );

    const result = await graphql<{
      updateProject: { name: string; description: string; public: boolean };
    }>(
      request,
      mutations.updateProject,
      {
        slug,
        input: {
          name: "Updated Name",
          description: "Updated description",
          public: true,
        },
      },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.updateProject.name).toBe("Updated Name");
    expect(result.data?.updateProject.description).toBe("Updated description");
    expect(result.data?.updateProject.public).toBe(true);
  });

  test("can partially update a project", async ({ request }) => {
    const slug = uniqueSlug("partial");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Original", description: "Keep this" } },
      testToken
    );

    // Only update name
    const result = await graphql<{
      updateProject: { name: string; description: string };
    }>(
      request,
      mutations.updateProject,
      { slug, input: { name: "New Name Only" } },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.updateProject.name).toBe("New Name Only");
    expect(result.data?.updateProject.description).toBe("Keep this");
  });

  test("can delete a project", async ({ request }) => {
    const slug = uniqueSlug("delete");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "To Delete" } },
      testToken
    );

    const result = await graphql<{ deleteProject: boolean }>(
      request,
      mutations.deleteProject,
      { slug },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.deleteProject).toBe(true);

    // Verify project no longer exists
    const checkResult = await graphql<{ project: null }>(
      request,
      queries.project,
      { slug },
      testToken
    );

    expect(checkResult.data?.project).toBeNull();
  });

  test("returns null for non-existent project", async ({ request }) => {
    const result = await graphql<{ project: null }>(
      request,
      queries.project,
      { slug: "non-existent-project-slug" },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.project).toBeNull();
  });

  test("projects are user-scoped", async ({ request }) => {
    const slug = uniqueSlug("scoped");

    // Create project with first user
    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "User 1 Project" } },
      testToken
    );

    // Try to access with different user
    const otherUserToken = createTestToken({
      id: "other-user-456",
      email: "other@example.com",
      name: "Other User",
    });

    const result = await graphql<{ project: null }>(
      request,
      queries.project,
      { slug },
      otherUserToken
    );

    // Other user should not see this project
    expect(result.data?.project).toBeNull();
  });
});
