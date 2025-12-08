import { test, expect } from "@playwright/test";
import { graphql, createTestToken, uniqueSlug, queries, mutations } from "./helpers";

test.describe("Thresholds", () => {
  const testToken = createTestToken({
    id: "threshold-test-user",
    email: "threshold-test@example.com",
    name: "Threshold Test User",
  });

  test("can create a global threshold (no branch/testbed)", async ({ request }) => {
    const slug = uniqueSlug("threshold-global");

    // Create project
    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Global Threshold Test" } },
      testToken
    );

    // Get the default latency measure ID
    const projectResult = await graphql<{
      project: { measures: Array<{ id: string; name: string }> };
    }>(request, queries.project, { slug }, testToken);

    const latencyMeasure = projectResult.data?.project.measures.find(
      (m) => m.name === "latency"
    );
    expect(latencyMeasure).toBeDefined();

    // Create threshold
    const result = await graphql<{
      createThreshold: {
        id: string;
        branchId: string | null;
        testbedId: string | null;
        measureId: string;
        upperBoundary: number;
        lowerBoundary: number | null;
        minSampleSize: number;
      };
    }>(
      request,
      mutations.createThreshold,
      {
        input: {
          projectSlug: slug,
          measureId: latencyMeasure!.id,
          upperBoundary: 10.0,
          minSampleSize: 3,
        },
      },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.createThreshold).toBeDefined();
    expect(result.data?.createThreshold.branchId).toBeNull();
    expect(result.data?.createThreshold.testbedId).toBeNull();
    expect(result.data?.createThreshold.upperBoundary).toBe(10.0);
    expect(result.data?.createThreshold.minSampleSize).toBe(3);
  });

  test("can create a branch-specific threshold", async ({ request }) => {
    const slug = uniqueSlug("threshold-branch");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Branch Threshold Test" } },
      testToken
    );

    // Create a report to create the branch
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

    // Get branch and measure IDs
    const projectResult = await graphql<{
      project: {
        branches: Array<{ id: string; name: string }>;
        measures: Array<{ id: string; name: string }>;
      };
    }>(request, queries.project, { slug }, testToken);

    const mainBranch = projectResult.data?.project.branches.find(
      (b) => b.name === "main"
    );
    const latencyMeasure = projectResult.data?.project.measures.find(
      (m) => m.name === "latency"
    );

    // Create branch-specific threshold
    const result = await graphql<{
      createThreshold: { branchId: string; testbedId: string | null };
    }>(
      request,
      mutations.createThreshold,
      {
        input: {
          projectSlug: slug,
          branchId: mainBranch!.id,
          measureId: latencyMeasure!.id,
          upperBoundary: 5.0,
        },
      },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.createThreshold.branchId).toBe(mainBranch!.id);
    expect(result.data?.createThreshold.testbedId).toBeNull();
  });

  test("can create a testbed-specific threshold", async ({ request }) => {
    const slug = uniqueSlug("threshold-testbed");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Testbed Threshold Test" } },
      testToken
    );

    // Create a report to create the testbed
    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "ci-linux",
          metrics: [{ benchmark: "test", measure: "latency", value: 100 }],
        },
      },
      testToken
    );

    const projectResult = await graphql<{
      project: {
        testbeds: Array<{ id: string; name: string }>;
        measures: Array<{ id: string; name: string }>;
      };
    }>(request, queries.project, { slug }, testToken);

    const testbed = projectResult.data?.project.testbeds.find(
      (t) => t.name === "ci-linux"
    );
    const latencyMeasure = projectResult.data?.project.measures.find(
      (m) => m.name === "latency"
    );

    const result = await graphql<{
      createThreshold: { testbedId: string };
    }>(
      request,
      mutations.createThreshold,
      {
        input: {
          projectSlug: slug,
          testbedId: testbed!.id,
          measureId: latencyMeasure!.id,
          upperBoundary: 15.0,
        },
      },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.createThreshold.testbedId).toBe(testbed!.id);
  });

  test("can update a threshold", async ({ request }) => {
    const slug = uniqueSlug("threshold-update");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Update Threshold Test" } },
      testToken
    );

    const projectResult = await graphql<{
      project: { measures: Array<{ id: string }> };
    }>(request, queries.project, { slug }, testToken);

    const measureId = projectResult.data?.project.measures[0].id;

    // Create threshold
    const createResult = await graphql<{
      createThreshold: { id: string };
    }>(
      request,
      mutations.createThreshold,
      {
        input: {
          projectSlug: slug,
          measureId,
          upperBoundary: 10.0,
          lowerBoundary: 5.0,
          minSampleSize: 2,
        },
      },
      testToken
    );

    const thresholdId = createResult.data?.createThreshold.id;

    // Update threshold
    const result = await graphql<{
      updateThreshold: {
        upperBoundary: number;
        lowerBoundary: number;
        minSampleSize: number;
      };
    }>(
      request,
      mutations.updateThreshold,
      {
        id: thresholdId,
        input: {
          upperBoundary: 20.0,
          lowerBoundary: 10.0,
          minSampleSize: 5,
        },
      },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.updateThreshold.upperBoundary).toBe(20.0);
    expect(result.data?.updateThreshold.lowerBoundary).toBe(10.0);
    expect(result.data?.updateThreshold.minSampleSize).toBe(5);
  });

  test("can partially update a threshold", async ({ request }) => {
    const slug = uniqueSlug("threshold-partial");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Partial Update Test" } },
      testToken
    );

    const projectResult = await graphql<{
      project: { measures: Array<{ id: string }> };
    }>(request, queries.project, { slug }, testToken);

    const measureId = projectResult.data?.project.measures[0].id;

    const createResult = await graphql<{
      createThreshold: { id: string };
    }>(
      request,
      mutations.createThreshold,
      {
        input: {
          projectSlug: slug,
          measureId,
          upperBoundary: 10.0,
          minSampleSize: 3,
        },
      },
      testToken
    );

    // Only update upperBoundary
    const result = await graphql<{
      updateThreshold: { upperBoundary: number; minSampleSize: number };
    }>(
      request,
      mutations.updateThreshold,
      {
        id: createResult.data?.createThreshold.id,
        input: { upperBoundary: 25.0 },
      },
      testToken
    );

    expect(result.data?.updateThreshold.upperBoundary).toBe(25.0);
    expect(result.data?.updateThreshold.minSampleSize).toBe(3); // unchanged
  });

  test("can delete a threshold", async ({ request }) => {
    const slug = uniqueSlug("threshold-delete");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Delete Threshold Test" } },
      testToken
    );

    const projectResult = await graphql<{
      project: { measures: Array<{ id: string }> };
    }>(request, queries.project, { slug }, testToken);

    const measureId = projectResult.data?.project.measures[0].id;

    const createResult = await graphql<{
      createThreshold: { id: string };
    }>(
      request,
      mutations.createThreshold,
      {
        input: { projectSlug: slug, measureId, upperBoundary: 10.0 },
      },
      testToken
    );

    const thresholdId = createResult.data?.createThreshold.id;

    const result = await graphql<{ deleteThreshold: boolean }>(
      request,
      mutations.deleteThreshold,
      { id: thresholdId },
      testToken
    );

    expect(result.errors).toBeUndefined();
    expect(result.data?.deleteThreshold).toBe(true);

    // Verify threshold is deleted
    const verifyResult = await graphql<{
      project: { thresholds: Array<{ id: string }> };
    }>(request, queries.project, { slug }, testToken);

    const found = verifyResult.data?.project.thresholds.find(
      (t) => t.id === thresholdId
    );
    expect(found).toBeUndefined();
  });

  test("thresholds appear in project query", async ({ request }) => {
    const slug = uniqueSlug("threshold-list");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "List Thresholds Test" } },
      testToken
    );

    const projectResult = await graphql<{
      project: { measures: Array<{ id: string }> };
    }>(request, queries.project, { slug }, testToken);

    const measureId = projectResult.data?.project.measures[0].id;

    // Create two thresholds
    await graphql(
      request,
      mutations.createThreshold,
      { input: { projectSlug: slug, measureId, upperBoundary: 10.0 } },
      testToken
    );
    await graphql(
      request,
      mutations.createThreshold,
      { input: { projectSlug: slug, measureId, upperBoundary: 20.0 } },
      testToken
    );

    const result = await graphql<{
      project: { thresholds: Array<{ upperBoundary: number }> };
    }>(request, queries.project, { slug }, testToken);

    expect(result.data?.project.thresholds.length).toBe(2);
  });

  test("default minSampleSize is 2", async ({ request }) => {
    const slug = uniqueSlug("threshold-default");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Default MinSampleSize Test" } },
      testToken
    );

    const projectResult = await graphql<{
      project: { measures: Array<{ id: string }> };
    }>(request, queries.project, { slug }, testToken);

    const measureId = projectResult.data?.project.measures[0].id;

    const result = await graphql<{
      createThreshold: { minSampleSize: number };
    }>(
      request,
      mutations.createThreshold,
      {
        input: {
          projectSlug: slug,
          measureId,
          upperBoundary: 10.0,
          // No minSampleSize specified
        },
      },
      testToken
    );

    expect(result.data?.createThreshold.minSampleSize).toBe(2);
  });
});
