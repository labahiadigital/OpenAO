import { env } from "$env/dynamic/private";

const DEFAULT_API_BASE_URL = "http://localhost:7667/api";

export function getApiBaseUrl(): string {
  return env.API_BASE_URL?.trim() || DEFAULT_API_BASE_URL;
}
