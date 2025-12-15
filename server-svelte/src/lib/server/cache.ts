interface CacheEntry<T> {
  data: T;
  timestamp: number;
  ttl: number;
}

const cache = new Map<string, CacheEntry<unknown>>();
const DEFAULT_TTL = 60 * 1000; // 60 seconds

export function getCached<T>(key: string): T | null {
  const entry = cache.get(key);
  if (!entry) return null;

  if (Date.now() - entry.timestamp > entry.ttl) {
    cache.delete(key);
    return null;
  }

  return entry.data as T;
}

export function setCache<T>(key: string, data: T, ttlMs?: number): void {
  cache.set(key, { data, timestamp: Date.now(), ttl: ttlMs ?? DEFAULT_TTL });
}

export function invalidateCache(pattern: string): void {
  for (const key of cache.keys()) {
    if (key.startsWith(pattern)) {
      cache.delete(key);
    }
  }
}

export function invalidateUserCache(userId: string): void {
  invalidateCache(`user:${userId}:`);
}

export function clearCache(): void {
  cache.clear();
}
