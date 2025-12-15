import type { PageServerLoad } from "./$types";
import { gqlQuery, PROJECTS_QUERY } from "$lib/server/graphql";

interface ProjectsData {
  projects: Array<{
    id: string;
    slug: string;
    name: string;
    description: string | null;
    public: boolean;
    createdAt: string;
  }>;
}

export const load: PageServerLoad = async ({ parent, locals }) => {
  await parent();

  const token = locals.session?.token;
  const userId = locals.session?.user.id;
  const data = await gqlQuery<ProjectsData>(PROJECTS_QUERY, {}, token, {
    cacheKey: userId ? `user:${userId}:projects` : undefined,
  });

  return {
    projects: data.projects,
  };
};
