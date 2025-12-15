import { PrismaClient } from "@prisma/client";

const DB_LATENCY_MS = process.env.DB_LATENCY_MS ? parseInt(process.env.DB_LATENCY_MS, 10) : 0;

const globalForPrisma = globalThis as unknown as { prisma: PrismaClient };

function createPrismaClient() {
  const client = new PrismaClient({
    log:
      process.env.NODE_ENV === "development"
        ? ["query", "error", "warn"]
        : ["error"],
  });

  if (DB_LATENCY_MS > 0) {
    client.$use(async (params, next) => {
      await new Promise((resolve) => setTimeout(resolve, DB_LATENCY_MS));
      return next(params);
    });
  }

  return client;
}

export const prisma = globalForPrisma.prisma || createPrismaClient();

if (process.env.NODE_ENV !== "production") globalForPrisma.prisma = prisma;

export async function getPrisma(): Promise<PrismaClient> {
  return prisma;
}
