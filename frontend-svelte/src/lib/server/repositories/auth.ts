import { query } from "../db";

export type AccountRow = {
  id: string;
  name: string;
  email: string;
  password_hash: string;
  created_at: Date;
};

export async function findAccountByEmail(
  email: string,
): Promise<AccountRow | null> {
  const result = await query<AccountRow>(
    "SELECT id, name, email, password_hash, created_at FROM accounts WHERE email = $1",
    [email],
  );
  return result.rows[0] ?? null;
}

export async function findAccountById(
  id: string,
): Promise<AccountRow | null> {
  const result = await query<AccountRow>(
    "SELECT id, name, email, password_hash, created_at FROM accounts WHERE id = $1",
    [id],
  );
  return result.rows[0] ?? null;
}

export async function createAccount(
  name: string,
  email: string,
  passwordHash: string,
): Promise<string> {
  const result = await query<{ id: string }>(
    "INSERT INTO accounts (name, email, password_hash) VALUES ($1, $2, $3) RETURNING id",
    [name, email, passwordHash],
  );
  return result.rows[0]!.id;
}
