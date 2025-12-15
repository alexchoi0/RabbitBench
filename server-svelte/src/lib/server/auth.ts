import * as crypto from "crypto";
import { betterAuth } from "better-auth";
import { prismaAdapter } from "better-auth/adapters/prisma";
import { prisma } from "$lib/server/db/prisma";
import { env } from "$env/dynamic/private";

export const DEV_MODE = env.DEV_MODE === "true" || env.DEV_MODE === "1";

function createDevSession() {
  const user = {
    id: "00000000-0000-0000-0000-000000000000",
    email: "dev@localhost",
    name: "Dev User",
    image: null,
  };
  return {
    user,
    token: createJwt({ sub: user.id, email: user.email, name: user.name || undefined }),
  };
}

export const auth = betterAuth({
  database: prismaAdapter(prisma, {
    provider: "postgresql",
  }),
  secret: env.BETTER_AUTH_SECRET,
  baseURL: env.BETTER_AUTH_URL || env.PUBLIC_APP_URL,
  trustedOrigins: ["http://localhost:5173"],
  emailAndPassword: {
    enabled: true,
  },
  socialProviders: {
    github: {
      clientId: env.GITHUB_CLIENT_ID || "",
      clientSecret: env.GITHUB_CLIENT_SECRET || "",
    },
  },
  session: {
    expiresIn: 60 * 60 * 24 * 7,
    updateAge: 60 * 60 * 24,
  },
  user: {
    additionalFields: {
      image: {
        type: "string",
        required: false,
      },
    },
  },
});

export async function getSessionFromRequest(request: Request) {
  if (DEV_MODE) {
    return createDevSession();
  }

  const session = await auth.api.getSession({
    headers: request.headers,
  });

  if (!session) return null;

  const user = {
    id: session.user.id,
    email: session.user.email,
    name: session.user.name,
    image: session.user.image ?? null,
  };

  return {
    user,
    token: createJwt({ sub: user.id, email: user.email, name: user.name || undefined }),
  };
}

export interface JwtClaims {
  sub: string;
  email: string;
  name?: string;
  picture?: string;
  iat: number;
  exp: number;
}

export function createJwt(payload: Omit<JwtClaims, "iat" | "exp">): string {
  const secret = env.BETTER_AUTH_SECRET || "";
  const now = Math.floor(Date.now() / 1000);
  const claims: JwtClaims = {
    ...payload,
    iat: now,
    exp: now + 60 * 60,
  };

  const header = Buffer.from(JSON.stringify({ alg: "HS256", typ: "JWT" })).toString("base64url");
  const payloadEncoded = Buffer.from(JSON.stringify(claims)).toString("base64url");
  const signatureInput = `${header}.${payloadEncoded}`;
  const signature = crypto
    .createHmac("sha256", secret)
    .update(signatureInput)
    .digest("base64url");

  return `${header}.${payloadEncoded}.${signature}`;
}

export function verifyJwt(token: string): JwtClaims | null {
  try {
    const parts = token.split(".");
    if (parts.length !== 3) return null;

    const [headerEncoded, payloadEncoded, signatureProvided] = parts;

    const signatureInput = `${headerEncoded}.${payloadEncoded}`;
    const secret = env.BETTER_AUTH_SECRET || "";
    const expectedSignature = crypto
      .createHmac("sha256", secret)
      .update(signatureInput)
      .digest("base64url");

    if (signatureProvided !== expectedSignature) return null;

    const payload = JSON.parse(
      Buffer.from(payloadEncoded, "base64url").toString()
    ) as JwtClaims;

    if (payload.exp && payload.exp < Math.floor(Date.now() / 1000)) {
      return null;
    }

    return payload;
  } catch {
    return null;
  }
}

