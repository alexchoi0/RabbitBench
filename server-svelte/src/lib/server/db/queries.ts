import { prisma } from "./prisma";

export async function ensureUser(user: {
  id: string;
  email: string;
  name?: string | null;
  image?: string | null;
}) {
  return prisma.user.upsert({
    where: { email: user.email },
    create: {
      id: user.id,
      email: user.email,
      name: user.name,
      image: user.image,
    },
    update: {
      name: user.name,
      image: user.image,
    },
  });
}
