import { randomUUID } from "crypto";

export function generateStoragePath(projectId: string, fileName: string): string {
  const uuid = randomUUID();
  return `projects/${projectId}/flamegraphs/${uuid}_${fileName}`;
}

export async function createSignedUploadUrl(storagePath: string): Promise<{
  signedUrl: string;
  token: string;
}> {
  const token = randomUUID();
  return {
    signedUrl: `/api/storage/upload?path=${encodeURIComponent(storagePath)}&token=${token}`,
    token,
  };
}

export async function createSignedReadUrl(storagePath: string): Promise<string> {
  return `/api/storage/read?path=${encodeURIComponent(storagePath)}`;
}
