import { test, expect } from "@playwright/test";
import { graphql, createTestToken, uniqueSlug, queries, mutations } from "./helpers";

test.describe("Alerts", () => {
  const testToken = createTestToken({
    id: "alert-test-user",
    email: "alert-test@example.com",
    name: "Alert Test User",
  });

  test("threshold violation creates an alert", async ({ request }) => {
    const slug = uniqueSlug("alert-violation");

    // Create project
    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Alert Violation Test" } },
      testToken
    );

    // Get measure ID
    const projectResult = await graphql<{
      project: { measures: Array<{ id: string }> };
    }>(request, queries.project, { slug }, testToken);

    const measureId = projectResult.data?.project.measures[0].id;

    // Create threshold: alert if value increases by more than 10%
    await graphql(
      request,
      mutations.createThreshold,
      {
        input: {
          projectSlug: slug,
          measureId,
          upperBoundary: 10.0, // 10% increase triggers alert
          minSampleSize: 2,
        },
      },
      testToken
    );

    // Create baseline reports (need at least minSampleSize = 2)
    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "local",
          metrics: [{ benchmark: "bench1", measure: "latency", value: 100 }],
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
          branch: "main",
          testbed: "local",
          metrics: [{ benchmark: "bench1", measure: "latency", value: 100 }],
        },
      },
      testToken
    );

    // Create a report with 50% increase (should trigger alert)
    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "local",
          metrics: [{ benchmark: "bench1", measure: "latency", value: 150 }], // 50% increase
        },
      },
      testToken
    );

    // Check for alerts
    const alertsResult = await graphql<{
      alerts: Array<{
        id: string;
        percentChange: number;
        baselineValue: number;
        status: string;
        metric: { benchmark: { name: string } };
      }>;
    }>(request, queries.alerts, { projectSlug: slug }, testToken);

    expect(alertsResult.errors).toBeUndefined();
    expect(alertsResult.data?.alerts.length).toBeGreaterThan(0);

    const alert = alertsResult.data?.alerts[0];
    expect(alert?.status).toBe("active");
    expect(alert?.percentChange).toBeCloseTo(50, 0); // ~50% increase
    expect(alert?.baselineValue).toBeCloseTo(100, 0); // average of baseline
    expect(alert?.metric.benchmark.name).toBe("bench1");
  });

  test("no alert when within threshold bounds", async ({ request }) => {
    const slug = uniqueSlug("alert-within");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Within Bounds Test" } },
      testToken
    );

    const projectResult = await graphql<{
      project: { measures: Array<{ id: string }> };
    }>(request, queries.project, { slug }, testToken);

    const measureId = projectResult.data?.project.measures[0].id;

    // Create threshold with 50% upper boundary
    await graphql(
      request,
      mutations.createThreshold,
      {
        input: {
          projectSlug: slug,
          measureId,
          upperBoundary: 50.0,
          minSampleSize: 2,
        },
      },
      testToken
    );

    // Create baseline
    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "local",
          metrics: [{ benchmark: "bench1", measure: "latency", value: 100 }],
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
          branch: "main",
          testbed: "local",
          metrics: [{ benchmark: "bench1", measure: "latency", value: 100 }],
        },
      },
      testToken
    );

    // Only 20% increase - within threshold
    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "local",
          metrics: [{ benchmark: "bench1", measure: "latency", value: 120 }],
        },
      },
      testToken
    );

    const alertsResult = await graphql<{ alerts: unknown[] }>(
      request,
      queries.alerts,
      { projectSlug: slug },
      testToken
    );

    expect(alertsResult.data?.alerts.length).toBe(0);
  });

  test("no alert when insufficient samples", async ({ request }) => {
    const slug = uniqueSlug("alert-samples");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Insufficient Samples Test" } },
      testToken
    );

    const projectResult = await graphql<{
      project: { measures: Array<{ id: string }> };
    }>(request, queries.project, { slug }, testToken);

    const measureId = projectResult.data?.project.measures[0].id;

    // Require 5 samples for baseline
    await graphql(
      request,
      mutations.createThreshold,
      {
        input: {
          projectSlug: slug,
          measureId,
          upperBoundary: 10.0,
          minSampleSize: 5,
        },
      },
      testToken
    );

    // Only create 2 baseline reports (need 5)
    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "local",
          metrics: [{ benchmark: "bench1", measure: "latency", value: 100 }],
        },
      },
      testToken
    );

    // Big increase but insufficient samples
    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "local",
          metrics: [{ benchmark: "bench1", measure: "latency", value: 500 }],
        },
      },
      testToken
    );

    const alertsResult = await graphql<{ alerts: unknown[] }>(
      request,
      queries.alerts,
      { projectSlug: slug },
      testToken
    );

    // No alert because we don't have enough samples
    expect(alertsResult.data?.alerts.length).toBe(0);
  });

  test("can dismiss an alert", async ({ request }) => {
    const slug = uniqueSlug("alert-dismiss");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Dismiss Alert Test" } },
      testToken
    );

    const projectResult = await graphql<{
      project: { measures: Array<{ id: string }> };
    }>(request, queries.project, { slug }, testToken);

    const measureId = projectResult.data?.project.measures[0].id;

    await graphql(
      request,
      mutations.createThreshold,
      {
        input: {
          projectSlug: slug,
          measureId,
          upperBoundary: 10.0,
          minSampleSize: 2,
        },
      },
      testToken
    );

    // Create baseline and violation
    for (let i = 0; i < 2; i++) {
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
    }

    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "local",
          metrics: [{ benchmark: "test", measure: "latency", value: 200 }],
        },
      },
      testToken
    );

    // Get the alert
    const alertsResult = await graphql<{
      alerts: Array<{ id: string; status: string }>;
    }>(request, queries.alerts, { projectSlug: slug }, testToken);

    const alertId = alertsResult.data?.alerts[0].id;
    expect(alertId).toBeDefined();

    // Dismiss it
    const dismissResult = await graphql<{
      dismissAlert: { id: string; status: string };
    }>(request, mutations.dismissAlert, { id: alertId }, testToken);

    expect(dismissResult.errors).toBeUndefined();
    expect(dismissResult.data?.dismissAlert.status).toBe("dismissed");

    // Verify it's dismissed when querying active alerts
    const activeAlertsResult = await graphql<{ alerts: unknown[] }>(
      request,
      queries.alerts,
      { projectSlug: slug, status: "active" },
      testToken
    );

    expect(activeAlertsResult.data?.alerts.length).toBe(0);
  });

  test("can filter alerts by status", async ({ request }) => {
    const slug = uniqueSlug("alert-filter");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Filter Alerts Test" } },
      testToken
    );

    const projectResult = await graphql<{
      project: { measures: Array<{ id: string }> };
    }>(request, queries.project, { slug }, testToken);

    const measureId = projectResult.data?.project.measures[0].id;

    await graphql(
      request,
      mutations.createThreshold,
      {
        input: {
          projectSlug: slug,
          measureId,
          upperBoundary: 10.0,
          minSampleSize: 2,
        },
      },
      testToken
    );

    // Create multiple alerts
    for (let i = 0; i < 2; i++) {
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
    }

    // Create two violations
    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "local",
          metrics: [{ benchmark: "test", measure: "latency", value: 200 }],
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
          branch: "main",
          testbed: "local",
          metrics: [{ benchmark: "test", measure: "latency", value: 300 }],
        },
      },
      testToken
    );

    // Get all alerts
    const allAlertsResult = await graphql<{
      alerts: Array<{ id: string }>;
    }>(request, queries.alerts, { projectSlug: slug }, testToken);

    expect(allAlertsResult.data?.alerts.length).toBeGreaterThanOrEqual(2);

    // Dismiss one
    const firstAlertId = allAlertsResult.data?.alerts[0].id;
    await graphql(
      request,
      mutations.dismissAlert,
      { id: firstAlertId },
      testToken
    );

    // Filter by active
    const activeAlertsResult = await graphql<{ alerts: unknown[] }>(
      request,
      queries.alerts,
      { projectSlug: slug, status: "active" },
      testToken
    );

    // Filter by dismissed
    const dismissedAlertsResult = await graphql<{ alerts: unknown[] }>(
      request,
      queries.alerts,
      { projectSlug: slug, status: "dismissed" },
      testToken
    );

    expect(activeAlertsResult.data?.alerts.length).toBeGreaterThanOrEqual(1);
    expect(dismissedAlertsResult.data?.alerts.length).toBe(1);
  });

  test("lower boundary threshold triggers alert on decrease", async ({ request }) => {
    const slug = uniqueSlug("alert-lower");

    await graphql(
      request,
      mutations.createProject,
      { input: { slug, name: "Lower Boundary Test" } },
      testToken
    );

    const projectResult = await graphql<{
      project: { measures: Array<{ id: string }> };
    }>(request, queries.project, { slug }, testToken);

    const measureId = projectResult.data?.project.measures[0].id;

    // Alert if value DECREASES by more than 20%
    await graphql(
      request,
      mutations.createThreshold,
      {
        input: {
          projectSlug: slug,
          measureId,
          lowerBoundary: 20.0, // Alert if decrease > 20%
          minSampleSize: 2,
        },
      },
      testToken
    );

    // Baseline
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
          branch: "main",
          testbed: "local",
          metrics: [{ benchmark: "test", measure: "latency", value: 100 }],
        },
      },
      testToken
    );

    // 50% decrease - should trigger lower boundary alert
    await graphql(
      request,
      mutations.createReport,
      {
        input: {
          projectSlug: slug,
          branch: "main",
          testbed: "local",
          metrics: [{ benchmark: "test", measure: "latency", value: 50 }],
        },
      },
      testToken
    );

    const alertsResult = await graphql<{
      alerts: Array<{ percentChange: number }>;
    }>(request, queries.alerts, { projectSlug: slug }, testToken);

    expect(alertsResult.data?.alerts.length).toBeGreaterThan(0);
    // Percent change should be negative (decrease)
    expect(alertsResult.data?.alerts[0].percentChange).toBeLessThan(0);
  });
});
