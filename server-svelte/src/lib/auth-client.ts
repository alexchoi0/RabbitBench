import { createAuthClient } from "better-auth/svelte";

export const authClient = createAuthClient({
  baseURL: import.meta.env.VITE_AUTH_URL || "",
});

export const { signIn, signOut, signUp, useSession } = authClient;
