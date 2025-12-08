import { test, expect } from "@playwright/test";
import { graphql, createTestToken, uniqueSlug, queries, mutations } from "./helpers";

test.describe("Performance Data Query", () => {
  const testToken = createTestToken({
    id: "perf-test-user",
    email: "perf-test@example.com",
    name: "Perf Test User",
  });

  test("can query performance data", async ({ request }) => {
    const slug = uniqueSlug("perf-query");

    // Setup: Create project with reports
    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Perf Query Test" } },
      testToken
    );

    // Create several reports with metrics
    for (let i = 0; i < 5; i++) {
      await graphql(
        request,
        mutations.createReport,
        {
          input: {
            projectSlug: slug,
            branch: "main",
            testbed: "linux",
            gitHash: `hash${i}`,
            metrics: [
              { benchmark: "fib/10", measure: "latency", value: 100 + i * 10 },
            ],
          },
        },
        testToken
      );
    }

    // Get IDs for query
    const projectResult = await graphql<{
      project: {
        benchmarks: Array<{ id: string; name: string }>;
        branches: Array<{ id: string; name: string }>;
        testbeds: Array<{ id: string; name: string }>;
        measures: Array<{ id: string; name: string }>;
      };
    }>(request, queries.project, { slug }, testToken);

    const benchmark = projectResult.data?.project.benchmarks.find(
      (b) => b.name === "fib/10"
    );
    const branch = projectResult.data?.project.branches.find(
      (b) => b.name === "main"
    );
    const testbed = projectResult.data?.project.testbeds.find(
      (t) => t.name === "linux"
    );
    const measure = projectResult.data?.project.measures.find(
      (m) => m.name === "latency"
    );

    // Query perf data
    const result = await graphql<{
      perf: {
        series: Array<{
          benchmark: { name: string };
          branch: { name: string };
          testbed: { name: string };
          measure: { name: string };
          data: Array<{ x: string; y: number; gitHash: string }>;
        }>;
      };
    }>(
      request,
      queries.perf,
      {
        projectSlug: slug,
        benchmarks: [benchmark!.id],
        branches: [branch!.id],
        testbeds: [testbed!.id],
        measures: [measure!.id],
      },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.perf.series).toHaveLength(1);

    const series = result.data?.perf.series[0];
    expect(series?.benchmark.name).toBe("fib/10");
    expect(series?.branch.name).toBe("main");
    expect(series?.testbed.name).toBe("linux");
    expect(series?.measure.name).toBe("latency");
    expect(series?.data).toHaveLength(5);

    // Data should be ordered by time (ascending)
    expect(series?.data[0].y).toBe(100);
    expect(series?.data[4].y).toBe(140);
  });

  test("returns multiple series for multiple benchmarks", async ({ request }) => {
    const slug = uniqueSlug("perf-multi");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Multi Series Test" } },
      testToken
    );

    // Create reports with multiple benchmarks
    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "local",
          metrics: [
            { benchmark: "bench1", measure: "latency", value: 100 },
            { benchmark: "bench2", measure: "latency", value: 200 },
          ],
        },
      },
      testToken
    );

    const projectResult = await graphql<{
      project: {
        benchmarks: Array<{ id: string }>;
        branches: Array<{ id: string }>;
        testbeds: Array<{ id: string }>;
        measures: Array<{ id: string }>;
      };
    }>(request, queries.project, { slug }, testToken);

    const result = await graphql<{
      perf: { series: Array<{ benchmark: { name: string } }> };
    }>(
      request,
      queries.perf,
      {
        projectSlug: slug,
        benchmarks: projectResult.data?.project.benchmarks.map((b) => b.id),
        branches: projectResult.data?.project.branches.map((b) => b.id),
        testbeds: projectResult.data?.project.testbeds.map((t) => t.id),
        measures: projectResult.data?.project.measures.map((m) => m.id),
      },
      testToken
    );

    expect(result.data?.perf.series.length).toBe(2);
    const benchmarkNames = result.data?.perf.series.map((s) => s.benchmark.name);
    expect(benchmarkNames).toContain("bench1");
    expect(benchmarkNames).toContain("bench2");
  });

  test("returns multiple series for multiple branches", async ({ request }) => {
    const slug = uniqueSlug("perf-branches");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Multi Branch Test" } },
      testToken
    );

    // Create reports on different branches
    await graphql(
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

    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "develop",
          testbed: "local",
          metrics: [{ benchmark: "test", measure: "latency", value: 150 }],
        },
      },
      testToken
    );

    const projectResult = await graphql<{
      project: {
        benchmarks: Array<{ id: string }>;
        branches: Array<{ id: string }>;
        testbeds: Array<{ id: string }>;
        measures: Array<{ id: string }>;
      };
    }>(request, queries.project, { slug }, testToken);

    const result = await graphql<{
      perf: { series: Array<{ branch: { name: string }; data: Array<{ y: number }> }> };
    }>(
      request,
      queries.perf,
      {
        projectSlug: slug,
        benchmarks: projectResult.data?.project.benchmarks.map((b) => b.id),
        branches: projectResult.data?.project.branches.map((b) => b.id),
        testbeds: projectResult.data?.project.testbeds.map((t) => t.id),
        measures: projectResult.data?.project.measures.map((m) => m.id),
      },
      testToken
    );

    expect(result.data?.perf.series.length).toBe(2);

    const mainSeries = result.data?.perf.series.find((s) => s.branch.name === "main");
    const developSeries = result.data?.perf.series.find((s) => s.branch.name === "develop");

    expect(mainSeries?.data[0].y).toBe(100);
    expect(developSeries?.data[0].y).toBe(150);
  });

  test("returns empty series when no matching data", async ({ request }) => {
    const slug = uniqueSlug("perf-empty");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Empty Perf Test" } },
      testToken
    );

    // Create a report to get IDs
    await graphql(
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

    const projectResult = await graphql<{
      project: {
        benchmarks: Array<{ id: string }>;
        branches: Array<{ id: string }>;
        testbeds: Array<{ id: string }>;
        measures: Array<{ id: string }>;
      };
    }>(request, queries.project, { slug }, testToken);

    // Query with non-existent IDs (use UUIDs that don't exist)
    const result = await graphql<{
      perf: { series: unknown[] };
    }>(
      request,
      queries.perf,
      {
        projectSlug: slug,
        benchmarks: ["00000000-0000-0000-0000-000000000000"],
        branches: projectResult.data?.project.branches.map((b) => b.id),
        testbeds: projectResult.data?.project.testbeds.map((t) => t.id),
        measures: projectResult.data?.project.measures.map((m) => m.id),
      },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.perf.series).toHaveLength(0);
  });

  test("includes confidence intervals in data points", async ({ request }) => {
    const slug = uniqueSlug("perf-intervals");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Confidence Intervals Test" } },
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
            {
              benchmark: "test",
              measure: "latency",
              value: 100,
              lowerValue: 95,
              upperValue: 105,
            },
          ],
        },
      },
      testToken
    );

    const projectResult = await graphql<{
      project: {
        benchmarks: Array<{ id: string }>;
        branches: Array<{ id: string }>;
        testbeds: Array<{ id: string }>;
        measures: Array<{ id: string }>;
      };
    }>(request, queries.project, { slug }, testToken);

    const result = await graphql<{
      perf: {
        series: Array<{
          data: Array<{ y: number; lower: number; upper: number }>;
        }>;
      };
    }>(
      request,
      queries.perf,
      {
        projectSlug: slug,
        benchmarks: projectResult.data?.project.benchmarks.map((b) => b.id),
        branches: projectResult.data?.project.branches.map((b) => b.id),
        testbeds: projectResult.data?.project.testbeds.map((t) => t.id),
        measures: projectResult.data?.project.measures.map((m) => m.id),
      },
      testToken
    );

    const dataPoint = result.data?.perf.series[0].data[0];
    expect(dataPoint?.y).toBe(100);
    expect(dataPoint?.lower).toBe(95);
    expect(dataPoint?.upper).toBe(105);
  });

  test("includes git hash in data points", async ({ request }) => {
    const slug = uniqueSlug("perf-githash");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Git Hash Test" } },
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
          gitHash: "abc123def456789",
          metrics: [{ benchmark: "test", measure: "latency", value: 100 }],
        },
      },
      testToken
    );

    const projectResult = await graphql<{
      project: {
        benchmarks: Array<{ id: string }>;
        branches: Array<{ id: string }>;
        testbeds: Array<{ id: string }>;
        measures: Array<{ id: string }>;
      };
    }>(request, queries.project, { slug }, testToken);

    const result = await graphql<{
      perf: { series: Array<{ data: Array<{ gitHash: string }> }> };
    }>(
      request,
      queries.perf,
      {
        projectSlug: slug,
        benchmarks: projectResult.data?.project.benchmarks.map((b) => b.id),
        branches: projectResult.data?.project.branches.map((b) => b.id),
        testbeds: projectResult.data?.project.testbeds.map((t) => t.id),
        measures: projectResult.data?.project.measures.map((m) => m.id),
      },
      testToken
    );

    expect(result.data?.perf.series[0].data[0].gitHash).toBe("abc123def456789");
  });

  test("filters by single branch", async ({ request }) => {
    const slug = uniqueSlug("perf-filter-branch");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Filter Branch Test" } },
      testToken
    );

    // Create reports on multiple branches
    await graphql(
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

    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "feature",
          testbed: "local",
          metrics: [{ benchmark: "test", measure: "latency", value: 200 }],
        },
      },
      testToken
    );

    const projectResult = await graphql<{
      project: {
        benchmarks: Array<{ id: string }>;
        branches: Array<{ id: string; name: string }>;
        testbeds: Array<{ id: string }>;
        measures: Array<{ id: string }>;
      };
    }>(request, queries.project, { slug }, testToken);

    const mainBranch = projectResult.data?.project.branches.find(
      (b) => b.name === "main"
    );

    // Only query main branch
    const result = await graphql<{
      perf: { series: Array<{ branch: { name: string } }> };
    }>(
      request,
      queries.perf,
      {
        projectSlug: slug,
        benchmarks: projectResult.data?.project.benchmarks.map((b) => b.id),
        branches: [mainBranch!.id],
        testbeds: projectResult.data?.project.testbeds.map((t) => t.id),
        measures: projectResult.data?.project.measures.map((m) => m.id),
      },
      testToken
    );

    expect(result.data?.perf.series.length).toBe(1);
    expect(result.data?.perf.series[0].branch.name).toBe("main");
  });
});
