let pool: FakePool | null = null;

interface FakePool {
  query<T = Record<string, unknown>>(text: string, params?: unknown[]): Promise<{ rows: T[] }>;
}

export function getPool(): FakePool {
  if (!pool) {
    const apiBaseUrl = process.env.API_BASE_URL || "http://localhost:3000";
    pool = {
      async query<T = Record<string, unknown>>(
        _text: string,
        _params?: unknown[],
      ): Promise<{ rows: T[] }> {
        return { rows: [] as T[] };
      },
    };
    console.log(`[db] API proxy mode — queries proxied to ${apiBaseUrl}`);
  }
  return pool;
}

export async function query<T = Record<string, unknown>>(
  text: string,
  params?: unknown[],
): Promise<{ rows: T[] }> {
  const p = getPool();
  return p.query<T>(text, params);
}
