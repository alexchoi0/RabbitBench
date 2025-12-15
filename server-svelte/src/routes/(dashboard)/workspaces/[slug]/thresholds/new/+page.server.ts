import { error, fail, redirect } from "@sveltejs/kit";
import type { PageServerLoad, Actions } from "./$types";
import {
  gqlQuery,
  gqlMutate,
  PROJECT_FOR_THRESHOLD_QUERY,
  CREATE_THRESHOLD_MUTATION,
} from "$lib/server/graphql";

interface ProjectData {
  project: {
    id: string;
    slug: string;
    name: string;
    branches: Array<{ id: string; name: string }>;
    testbeds: Array<{ id: string; name: string }>;
    measures: Array<{ id: string; name: string; units: string | null }>;
  } | null;
}

export const load: PageServerLoad = async ({ params, parent, locals }) => {
  await parent();

  const token = locals.session?.token;
  const data = await gqlQuery<ProjectData>(
    PROJECT_FOR_THRESHOLD_QUERY,
    { slug: params.slug },
    token
  );

  if (!data.project) {
    error(404, "Workspace not found");
  }

  return { project: data.project };
};

export const actions: Actions = {
  create: async ({ request, params, locals }) => {
    if (!locals.session) {
      return fail(401, { error: "Not authenticated" });
    }

    const formData = await request.formData();
    const measureId = formData.get("measureId") as string;
    const branchId = formData.get("branchId") as string;
    const testbedId = formData.get("testbedId") as string;
    const upperBoundary = formData.get("upperBoundary") as string;
    const lowerBoundary = formData.get("lowerBoundary") as string;
    const minSampleSize = formData.get("minSampleSize") as string;

    if (!measureId) {
      return fail(400, { error: "Please select a measure" });
    }

    if (!upperBoundary && !lowerBoundary) {
      return fail(400, { error: "Please set at least one boundary (upper or lower)" });
    }

    try {
      await gqlMutate(
        CREATE_THRESHOLD_MUTATION,
        {
          input: {
            projectSlug: params.slug,
            measureId,
            branchId: branchId && branchId !== "all" ? branchId : null,
            testbedId: testbedId && testbedId !== "all" ? testbedId : null,
            upperBoundary: upperBoundary ? parseFloat(upperBoundary) : null,
            lowerBoundary: lowerBoundary ? parseFloat(lowerBoundary) : null,
            minSampleSize: minSampleSize ? parseInt(minSampleSize, 10) : 2,
          },
        },
        locals.session.token
      );

      redirect(303, `/workspaces/${params.slug}?tab=thresholds`);
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to create threshold";
      return fail(400, { error: message });
    }
  },
};
