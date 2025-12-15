import type { Handle } from "@sveltejs/kit";
import { getSessionFromRequest } from "$lib/server/auth";

export const handle: Handle = async ({ event, resolve }) => {
  const session = await getSessionFromRequest(event.request);
  event.locals.session = session;

  return resolve(event);
};
