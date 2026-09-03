import { getApiBaseUrl } from "$lib/api-base-url";
import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = async ({ fetch }) => {
  const apiBase = getApiBaseUrl();

  try {
    const response = await fetch(`${apiBase}/ranking`);
    if (!response.ok) return { rankings: [] };
    const rankings = await response.json();
    return { rankings };
  } catch {
    return { rankings: [] };
  }
};
