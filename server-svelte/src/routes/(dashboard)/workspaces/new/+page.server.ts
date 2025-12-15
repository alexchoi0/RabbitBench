import { fail, redirect } from "@sveltejs/kit";
import type { Actions } from "./$types";
import { gqlMutate, CREATE_PROJECT_MUTATION } from "$lib/server/graphql";
import { invalidateCache } from "$lib/server/cache";

interface CreateProjectData {
  createProject: {
    id: string;
    slug: string;
  };
}

export const actions: Actions = {
  create: async ({ request, locals }) => {
    if (!locals.session) {
      return fail(401, { error: "Not authenticated" });
    }

    const formData = await request.formData();
    const slug = formData.get("slug") as string;
    const name = formData.get("name") as string;
    const description = formData.get("description") as string;

    if (!slug || !name) {
      return fail(400, { error: "Slug and name are required" });
    }

    if (!/^[a-z0-9-]+$/.test(slug)) {
      return fail(400, { error: "Slug must contain only lowercase letters, numbers, and hyphens" });
    }

    try {
      const data = await gqlMutate<CreateProjectData>(
        CREATE_PROJECT_MUTATION,
        {
          input: {
            slug,
            name,
            description: description || null,
          },
        },
        locals.session.token
      );

      // Invalidate projects list cache
      const userId = locals.session.user.id;
      invalidateCache(`user:${userId}:projects`);

      redirect(303, `/workspaces/${data.createProject.slug}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to create workspace";
      return fail(400, { error: message });
    }
  },
};
