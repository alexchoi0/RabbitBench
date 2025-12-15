import { redirect } from "@sveltejs/kit";
import type { LayoutServerLoad } from "./$types";
import { ensureUser } from "$lib/server/db/queries";

export const load: LayoutServerLoad = async ({ locals }) => {
  if (!locals.session) {
    redirect(303, "/login");
  }

  await ensureUser(locals.session.user);

  return {
    user: locals.session.user,
  };
};
