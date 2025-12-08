import { test, expect } from "@playwright/test";
import { graphql, createTestToken, uniqueSlug, queries, mutations } from "./helpers";

test.describe("GitHub Integration", () => {
  const testToken = createTestToken({
    id: "github-test-user",
    email: "github-test@example.com",
    name: "GitHub Test User",
  });

  test.describe("GitHub Settings", () => {
    test("new project has no GitHub settings by default", async ({ request }) => {
      const slug = uniqueSlug("github-default");

      await graphql(
        request,
        mutations.createProject,
        { input: { slug, name: "GitHub Default Test" } },
        testToken
      );

      const result = await graphql<{
        project: {
          githubRepo: string | null;
          githubPrComments: boolean;
          githubStatusChecks: boolean;
          hasGithubToken: boolean;
        };
      }>(request, queries.project, { slug }, testToken);

      expect(result.errors).toBeUndefined();
      expect(result.data?.project.githubRepo).toBeNull();
      expect(result.data?.project.githubPrComments).toBe(false);
      expect(result.data?.project.githubStatusChecks).toBe(false);
      expect(result.data?.project.hasGithubToken).toBe(false);
    });

    test("can update GitHub repo setting", async ({ request }) => {
      const slug = uniqueSlug("github-repo");

      await graphql(
        request,
        mutations.createProject,
        { input: { slug, name: "GitHub Repo Test" } },
        testToken
      );

      const result = await graphql<{
        updateGitHubSettings: {
          githubRepo: string;
          githubPrComments: boolean;
          githubStatusChecks: boolean;
          hasGithubToken: boolean;
        };
      }>(
        request,
        mutations.updateGitHubSettings,
        {
          slug,
          input: {
            githubRepo: "owner/repo",
          },
        },
        testToken
      );

      expect(result.errors).toBeUndefined();
      expect(result.data?.updateGitHubSettings.githubRepo).toBe("owner/repo");
      expect(result.data?.updateGitHubSettings.hasGithubToken).toBe(false);
    });

    test("can enable PR comments", async ({ request }) => {
      const slug = uniqueSlug("github-pr-comments");

      await graphql(
        request,
        mutations.createProject,
        { input: { slug, name: "GitHub PR Comments Test" } },
        testToken
      );

      const result = await graphql<{
        updateGitHubSettings: {
          githubRepo: string;
          githubPrComments: boolean;
          githubStatusChecks: boolean;
        };
      }>(
        request,
        mutations.updateGitHubSettings,
        {
          slug,
          input: {
            githubRepo: "owner/repo",
            githubPrComments: true,
          },
        },
        testToken
      );

      expect(result.errors).toBeUndefined();
      expect(result.data?.updateGitHubSettings.githubPrComments).toBe(true);
      expect(result.data?.updateGitHubSettings.githubStatusChecks).toBe(false);
    });

    test("can enable status checks", async ({ request }) => {
      const slug = uniqueSlug("github-status");

      await graphql(
        request,
        mutations.createProject,
        { input: { slug, name: "GitHub Status Test" } },
        testToken
      );

      const result = await graphql<{
        updateGitHubSettings: {
          githubStatusChecks: boolean;
        };
      }>(
        request,
        mutations.updateGitHubSettings,
        {
          slug,
          input: {
            githubRepo: "owner/repo",
            githubStatusChecks: true,
          },
        },
        testToken
      );

      expect(result.errors).toBeUndefined();
      expect(result.data?.updateGitHubSettings.githubStatusChecks).toBe(true);
    });

    test("can set GitHub token", async ({ request }) => {
      const slug = uniqueSlug("github-token");

      await graphql(
        request,
        mutations.createProject,
        { input: { slug, name: "GitHub Token Test" } },
        testToken
      );

      const result = await graphql<{
        updateGitHubSettings: {
          hasGithubToken: boolean;
        };
      }>(
        request,
        mutations.updateGitHubSettings,
        {
          slug,
          input: {
            githubToken: "ghp_test_token_12345",
          },
        },
        testToken
      );

      expect(result.errors).toBeUndefined();
      expect(result.data?.updateGitHubSettings.hasGithubToken).toBe(true);
    });

    test("can update all GitHub settings at once", async ({ request }) => {
      const slug = uniqueSlug("github-all");

      await graphql(
        request,
        mutations.createProject,
        { input: { slug, name: "GitHub All Settings Test" } },
        testToken
      );

      const result = await graphql<{
        updateGitHubSettings: {
          githubRepo: string;
          githubPrComments: boolean;
          githubStatusChecks: boolean;
          hasGithubToken: boolean;
        };
      }>(
        request,
        mutations.updateGitHubSettings,
        {
          slug,
          input: {
            githubRepo: "myorg/myrepo",
            githubToken: "ghp_secret_token",
            githubPrComments: true,
            githubStatusChecks: true,
          },
        },
        testToken
      );

      expect(result.errors).toBeUndefined();
      expect(result.data?.updateGitHubSettings.githubRepo).toBe("myorg/myrepo");
      expect(result.data?.updateGitHubSettings.githubPrComments).toBe(true);
      expect(result.data?.updateGitHubSettings.githubStatusChecks).toBe(true);
      expect(result.data?.updateGitHubSettings.hasGithubToken).toBe(true);
    });

    test("can clear GitHub repo", async ({ request }) => {
      const slug = uniqueSlug("github-clear");

      await graphql(
        request,
        mutations.createProject,
        { input: { slug, name: "GitHub Clear Test" } },
        testToken
      );

      // First set a repo
      await graphql(
        request,
        mutations.updateGitHubSettings,
        {
          slug,
          input: { githubRepo: "owner/repo" },
        },
        testToken
      );

      // Then clear it
      const result = await graphql<{
        updateGitHubSettings: {
          githubRepo: string | null;
        };
      }>(
        request,
        mutations.updateGitHubSettings,
        {
          slug,
          input: { githubRepo: "" },
        },
        testToken
      );

      expect(result.errors).toBeUndefined();
      expect(result.data?.updateGitHubSettings.githubRepo).toBeNull();
    });

    test("GitHub settings persist after update", async ({ request }) => {
      const slug = uniqueSlug("github-persist");

      await graphql(
        request,
        mutations.createProject,
        { input: { slug, name: "GitHub Persist Test" } },
        testToken
      );

      // Set GitHub settings
      await graphql(
        request,
        mutations.updateGitHubSettings,
        {
          slug,
          input: {
            githubRepo: "test/repo",
            githubPrComments: true,
            githubStatusChecks: true,
            githubToken: "ghp_token",
          },
        },
        testToken
      );

      // Fetch project and verify settings persisted
      const result = await graphql<{
        project: {
          githubRepo: string;
          githubPrComments: boolean;
          githubStatusChecks: boolean;
          hasGithubToken: boolean;
        };
      }>(request, queries.project, { slug }, testToken);

      expect(result.errors).toBeUndefined();
      expect(result.data?.project.githubRepo).toBe("test/repo");
      expect(result.data?.project.githubPrComments).toBe(true);
      expect(result.data?.project.githubStatusChecks).toBe(true);
      expect(result.data?.project.hasGithubToken).toBe(true);
    });

    test("cannot update GitHub settings for non-existent project", async ({ request }) => {
      const result = await graphql(
        request,
        mutations.updateGitHubSettings,
        {
          slug: "non-existent-project",
          input: { githubRepo: "owner/repo" },
        },
        testToken
      );

      expect(result.errors).toBeDefined();
      expect(result.errors?.[0].message).toContain("Project not found");
    });

    test("cannot update GitHub settings for another user's project", async ({ request }) => {
      const slug = uniqueSlug("github-other-user");

      // Create project with first user
      await graphql(
        request,
        mutations.createProject,
        { input: { slug, name: "GitHub Other User Test" } },
        testToken
      );

      // Try to update with different user
      const otherUserToken = createTestToken({
        id: "other-user-github",
        email: "other-github@example.com",
        name: "Other GitHub User",
      });

      const result = await graphql(
        request,
        mutations.updateGitHubSettings,
        {
          slug,
          input: { githubRepo: "hacker/repo" },
        },
        otherUserToken
      );

      expect(result.errors).toBeDefined();
      expect(result.errors?.[0].message).toContain("Project not found");
    });
  });

  test.describe("Reports with PR Number", () => {
    test("can create report with PR number", async ({ request }) => {
      const slug = uniqueSlug("report-pr");

      await graphql(
        request,
        mutations.createProject,
        { input: { slug, name: "Report PR Test" } },
        testToken
      );

      // Create report with prNumber (the mutation should accept it)
      const createReportWithPR = `
        mutation CreateReport($input: CreateReportInput!) {
          createReport(input: $input) {
            id
            gitHash
          }
        }
      `;

      const result = await graphql<{
        createReport: { id: string; gitHash: string };
      }>(
        request,
        createReportWithPR,
        {
          input: {
            projectSlug: slug,
            branch: "feature/pr-test",
            testbed: "ci",
            gitHash: "abc123",
            prNumber: 42,
            metrics: [{ benchmark: "test", measure: "latency", value: 100 }],
          },
        },
        testToken
      );

      expect(result.errors).toBeUndefined();
      expect(result.data?.createReport.id).toBeDefined();
    });

    test("report without PR number is valid", async ({ request }) => {
      const slug = uniqueSlug("report-no-pr");

      await graphql(
        request,
        mutations.createProject,
        { input: { slug, name: "Report No PR Test" } },
        testToken
      );

      const result = await graphql<{
        createReport: { id: string };
      }>(
        request,
        mutations.createReport,
        {
          input: {
            projectSlug: slug,
            branch: "main",
            testbed: "local",
            metrics: [{ benchmark: "test", measure: "latency", value: 100 }],
          },
        },
        testToken
      );

      expect(result.errors).toBeUndefined();
      expect(result.data?.createReport.id).toBeDefined();
    });
  });
});
