import { test, expect } from "@playwright/test";
import { graphql, createTestToken, uniqueSlug, queries, mutations } from "./helpers";

test.describe("Reports and Metrics", () => {
  const testToken = createTestToken({
    id: "report-test-user",
    email: "report-test@example.com",
    name: "Report Test User",
  });

  test("can create a report with metrics", async ({ request }) => {
    const slug = uniqueSlug("report");

    // Create project first
    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Report Test Project" } },
      testToken
    );

    const result = await graphql<{
      createReport: {
        id: string;
        gitHash: string;
        branch: { name: string };
        testbed: { name: string };
        metrics: Array<{
          value: number;
          benchmark: { name: string };
          measure: { name: string };
        }>;
      };
    }>(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "linux-ci",
          gitHash: "abc123def456",
          metrics: [
            {
              benchmark: "fibonacci/10",
              measure: "latency",
              value: 1234.5,
              lowerValue: 1200.0,
              upperValue: 1300.0,
            },
            {
              benchmark: "fibonacci/20",
              measure: "latency",
              value: 5678.9,
            },
          ],
        },
      },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.createReport).toBeDefined();
    expect(result.data?.createReport.gitHash).toBe("abc123def456");
    expect(result.data?.createReport.branch.name).toBe("main");
    expect(result.data?.createReport.testbed.name).toBe("linux-ci");
    expect(result.data?.createReport.metrics).toHaveLength(2);

    const fib10 = result.data?.createReport.metrics.find(
      (m) => m.benchmark.name === "fibonacci/10"
    );
    expect(fib10?.value).toBe(1234.5);
    expect(fib10?.measure.name).toBe("latency");
  });

  test("auto-creates branch on first report", async ({ request }) => {
    const slug = uniqueSlug("autobranch");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Auto Branch Test" } },
      testToken
    );

    // Create report with new branch
    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "feature/new-branch",
          testbed: "local",
          metrics: [{ benchmark: "test", measure: "latency", value: 100 }],
        },
      },
      testToken
    );

    // Verify branch was created
    const projectResult = await graphql<{
      project: { branches: Array<{ name: string }> };
    }>(request, queries.project, { slug }, testToken);

    const branches = projectResult.data?.project.branches.map((b) => b.name);
    expect(branches).toContain("feature/new-branch");
  });

  test("auto-creates testbed on first report", async ({ request }) => {
    const slug = uniqueSlug("autotestbed");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Auto Testbed Test" } },
      testToken
    );

    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "macos-arm64",
          metrics: [{ benchmark: "test", measure: "latency", value: 100 }],
        },
      },
      testToken
    );

    const projectResult = await graphql<{
      project: { testbeds: Array<{ name: string }> };
    }>(request, queries.project, { slug }, testToken);

    const testbeds = projectResult.data?.project.testbeds.map((t) => t.name);
    expect(testbeds).toContain("macos-arm64");
  });

  test("auto-creates benchmark on first report", async ({ request }) => {
    const slug = uniqueSlug("autobenchmark");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Auto Benchmark Test" } },
      testToken
    );

    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "local",
          metrics: [
            { benchmark: "new_benchmark_test", measure: "latency", value: 100 },
          ],
        },
      },
      testToken
    );

    const projectResult = await graphql<{
      project: { benchmarks: Array<{ name: string }> };
    }>(request, queries.project, { slug }, testToken);

    const benchmarks = projectResult.data?.project.benchmarks.map((b) => b.name);
    expect(benchmarks).toContain("new_benchmark_test");
  });

  test("auto-creates measure on first report", async ({ request }) => {
    const slug = uniqueSlug("automeasure");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Auto Measure Test" } },
      testToken
    );

    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "local",
          metrics: [
            { benchmark: "test", measure: "throughput", value: 1000 },
          ],
        },
      },
      testToken
    );

    const projectResult = await graphql<{
      project: { measures: Array<{ name: string }> };
    }>(request, queries.project, { slug }, testToken);

    const measures = projectResult.data?.project.measures.map((m) => m.name);
    expect(measures).toContain("throughput");
  });

  test("reports appear in recentReports", async ({ request }) => {
    const slug = uniqueSlug("recent");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Recent Reports Test" } },
      testToken
    );

    // Create multiple reports
    for (let i = 0; i < 3; i++) {
      await graphql(
        request,
        mutations.createReport,
        {
          input: {
            projectSlug: slug,
            branch: "main",
            testbed: "local",
            gitHash: `hash${i}`,
            metrics: [{ benchmark: "test", measure: "latency", value: 100 + i }],
          },
        },
        testToken
      );
    }

    const projectResult = await graphql<{
      project: { recentReports: Array<{ gitHash: string }> };
    }>(request, queries.project, { slug }, testToken);

    expect(projectResult.data?.project.recentReports.length).toBe(3);
    // Most recent first
    expect(projectResult.data?.project.recentReports[0].gitHash).toBe("hash2");
  });

  test("cannot create report for non-existent project", async ({ request }) => {
    const result = await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: "non-existent-project",
          branch: "main",
          testbed: "local",
          metrics: [{ benchmark: "test", measure: "latency", value: 100 }],
        },
      },
      testToken
    );

    expect(result.errors).toBeDefined();
    expect(result.errors?.[0].message).toContain("Project not found");
  });

  test("report without gitHash is valid", async ({ request }) => {
    const slug = uniqueSlug("nogithash");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "No Git Hash Test" } },
      testToken
    );

    const result = await graphql<{
      createReport: { gitHash: string | null };
    }>(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "local",
          // No gitHash
          metrics: [{ benchmark: "test", measure: "latency", value: 100 }],
        },
      },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.createReport.gitHash).toBeNull();
  });
});
