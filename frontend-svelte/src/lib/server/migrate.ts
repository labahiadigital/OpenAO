import fs from "node:fs";
import path from "node:path";
import { getPool } from "./db";

export async function runMigrations(schemaPath: string): Promise<void> {
  const pool = getPool();

  await pool.query(`
    CREATE TABLE IF NOT EXISTS _migrations (
      id SERIAL PRIMARY KEY,
      filename TEXT NOT NULL UNIQUE,
      applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    )
  `);

  const files = fs
    .readdirSync(schemaPath)
    .filter((f) => f.endsWith(".sql"))
    .sort();

  const applied = await pool.query<{ filename: string }>(
    "SELECT filename FROM _migrations ORDER BY id",
  );
  const appliedSet = new Set(applied.rows.map((r) => r.filename));

  for (const file of files) {
    if (appliedSet.has(file)) continue;

    const sql = fs.readFileSync(path.join(schemaPath, file), "utf-8");
    const client = await pool.connect();

    try {
      await client.query("BEGIN");
      await client.query(sql);
      await client.query("INSERT INTO _migrations (filename) VALUES ($1)", [file]);
      await client.query("COMMIT");
      console.log(`Migration applied: ${file}`);
    } catch (err) {
      await client.query("ROLLBACK");
      throw err;
    } finally {
      client.release();
    }
  }
}
