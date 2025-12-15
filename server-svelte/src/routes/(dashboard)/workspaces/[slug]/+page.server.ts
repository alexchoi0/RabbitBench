import { error } from "@sveltejs/kit";
import type { PageServerLoad } from "./$types";
import { gqlQuery, PROJECT_QUERY, ACTIVE_ALERTS_QUERY } from "$lib/server/graphql";

interface ProjectData {
  project: {
    id: string;
    slug: string;
    name: string;
    description: string | null;
    public: boolean;
    branches: Array<{ id: string; name: string }>;
    testbeds: Array<{ id: string; name: string }>;
    measures: Array<{ id: string; name: string; units: string | null }>;
    benchmarks: Array<{ id: string; name: string }>;
    reports: Array<{
      id: string;
      gitHash: string | null;
      prNumber: number | null;
      createdAt: string;
      branch: { id: string; name: string };
      testbed: { id: string; name: string };
    }>;
    thresholds: Array<{
      id: string;
      measureId: string;
      branchId: string | null;
      testbedId: string | null;
      upperBoundary: number | null;
      lowerBoundary: number | null;
      minSampleSize: number;
    }>;
  } | null;
}

interface AlertsData {
  project: {
    alerts: Array<{
      id: string;
      percentChange: number;
      metric: {
        benchmark: { name: string };
        measure: { name: string };
      };
    }>;
  } | null;
}

export const load: PageServerLoad = async ({ params, parent, locals }) => {
  await parent();

  const token = locals.session?.token;
  const userId = locals.session?.user.id;

  const [projectData, alertsData] = await Promise.all([
    gqlQuery<ProjectData>(PROJECT_QUERY, { slug: params.slug }, token, {
      cacheKey: userId ? `user:${userId}:project:${params.slug}` : undefined,
    }),
    gqlQuery<AlertsData>(ACTIVE_ALERTS_QUERY, { slug: params.slug }, token, {
      cacheKey: userId ? `user:${userId}:project:${params.slug}:alerts` : undefined,
    }),
  ]);

  if (!projectData.project) {
    error(404, "Workspace not found");
  }

  return {
    project: projectData.project,
    alerts: alertsData.project?.alerts || [],
  };
};
