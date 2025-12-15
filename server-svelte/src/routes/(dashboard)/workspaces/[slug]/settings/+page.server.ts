import { error, fail, redirect } from "@sveltejs/kit";
import type { PageServerLoad, Actions } from "./$types";
import {
  gqlQuery,
  gqlMutate,
  PROJECT_SETTINGS_QUERY,
  UPDATE_PROJECT_MUTATION,
  DELETE_PROJECT_MUTATION,
  UPDATE_GITHUB_SETTINGS_MUTATION,
  CREATE_API_TOKEN_MUTATION,
  REVOKE_API_TOKEN_MUTATION,
} from "$lib/server/graphql";
import { invalidateCache } from "$lib/server/cache";

interface ProjectSettingsData {
  project: {
    id: string;
    slug: string;
    name: string;
    description: string | null;
    public: boolean;
    githubRepo: string | null;
    githubPrComments: boolean;
    githubStatusChecks: boolean;
    hasGithubToken: boolean;
  } | null;
  apiTokens: Array<{
    id: string;
    name: string;
    lastUsedAt: string | null;
    createdAt: string;
  }>;
}

export const load: PageServerLoad = async ({ params, parent, locals }) => {
  await parent();

  const token = locals.session?.token;
  const userId = locals.session?.user.id;
  const data = await gqlQuery<ProjectSettingsData>(
    PROJECT_SETTINGS_QUERY,
    { slug: params.slug },
    token,
    {
      cacheKey: userId ? `user:${userId}:project:${params.slug}:settings` : undefined,
    }
  );

  if (!data.project) {
    error(404, "Workspace not found");
  }

  return {
    project: data.project,
    tokens: data.apiTokens,
  };
};

export const actions: Actions = {
  updateProject: async ({ request, params, locals }) => {
    if (!locals.session) {
      return fail(401, { error: "Not authenticated" });
    }

    const formData = await request.formData();
    const name = formData.get("name") as string;
    const description = formData.get("description") as string;
    const isPublic = formData.get("public") === "true";

    if (!name) {
      return fail(400, { error: "Name is required" });
    }

    try {
      await gqlMutate(
        UPDATE_PROJECT_MUTATION,
        {
          slug: params.slug,
          input: {
            name,
            description: description || null,
            public: isPublic,
          },
        },
        locals.session.token
      );
      // Invalidate project caches
      const userId = locals.session.user.id;
      invalidateCache(`user:${userId}:project:${params.slug}`);
      invalidateCache(`user:${userId}:projects`);
      return { success: true };
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to save";
      return fail(400, { error: message });
    }
  },

  deleteProject: async ({ params, locals }) => {
    if (!locals.session) {
      return fail(401, { error: "Not authenticated" });
    }

    try {
      await gqlMutate(
        DELETE_PROJECT_MUTATION,
        { slug: params.slug },
        locals.session.token
      );
      // Invalidate project caches
      const userId = locals.session.user.id;
      invalidateCache(`user:${userId}:project:${params.slug}`);
      invalidateCache(`user:${userId}:projects`);
      redirect(303, "/workspaces");
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to delete";
      return fail(400, { error: message });
    }
  },

  createToken: async ({ request, locals }) => {
    if (!locals.session) {
      return fail(401, { error: "Not authenticated" });
    }

    const formData = await request.formData();
    const name = formData.get("name") as string;

    if (!name) {
      return fail(400, { error: "Token name is required" });
    }

    try {
      const data = await gqlMutate<{
        createApiToken: {
          token: { id: string; name: string };
          secret: string;
        };
      }>(CREATE_API_TOKEN_MUTATION, { name }, locals.session.token);

      // Invalidate settings cache (contains token list)
      const userId = locals.session.user.id;
      invalidateCache(`user:${userId}:project:`);

      return {
        success: true,
        token: data.createApiToken.token,
        secret: data.createApiToken.secret,
      };
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to create token";
      return fail(400, { error: message });
    }
  },

  revokeToken: async ({ request, locals }) => {
    if (!locals.session) {
      return fail(401, { error: "Not authenticated" });
    }

    const formData = await request.formData();
    const tokenId = formData.get("tokenId") as string;

    if (!tokenId) {
      return fail(400, { error: "Token ID is required" });
    }

    try {
      await gqlMutate(REVOKE_API_TOKEN_MUTATION, { id: tokenId }, locals.session.token);
      // Invalidate settings cache (contains token list)
      const userId = locals.session.user.id;
      invalidateCache(`user:${userId}:project:`);
      return { success: true };
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to revoke token";
      return fail(400, { error: message });
    }
  },

  updateGitHub: async ({ request, params, locals }) => {
    if (!locals.session) {
      return fail(401, { error: "Not authenticated" });
    }

    const formData = await request.formData();
    const githubRepo = formData.get("githubRepo") as string;
    const githubToken = formData.get("githubToken") as string;
    const githubPrComments = formData.get("githubPrComments") === "true";
    const githubStatusChecks = formData.get("githubStatusChecks") === "true";

    try {
      await gqlMutate(
        UPDATE_GITHUB_SETTINGS_MUTATION,
        {
          slug: params.slug,
          input: {
            githubRepo: githubRepo || null,
            githubToken: githubToken || null,
            githubPrComments,
            githubStatusChecks,
          },
        },
        locals.session.token
      );
      // Invalidate project caches
      const userId = locals.session.user.id;
      invalidateCache(`user:${userId}:project:${params.slug}`);
      return { success: true };
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to save GitHub settings";
      return fail(400, { error: message });
    }
  },
};
