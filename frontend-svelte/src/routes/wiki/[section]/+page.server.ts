import { getApiBaseUrl } from "$lib/api-base-url";
import type { PageServerLoad } from "./$types";

export type WikiData = {
  items: { id: number; name: string; type: number; grhIndex: number }[];
  npcs: { id: number; name: string; hp: number; exp: number }[];
  spells: { id: number; name: string; manaRequired: number; type: number }[];
};

export const load: PageServerLoad = async ({ fetch, setHeaders }) => {
  const apiBase = getApiBaseUrl();

  try {
    const response = await fetch(`${apiBase}/wiki`);
    if (!response.ok) return { wiki: null };
    const wiki: WikiData = await response.json();

    setHeaders({
      "cache-control": "public, max-age=300, s-maxage=600, stale-while-revalidate=86400",
    });

    return { wiki };
  } catch {
    return { wiki: null };
  }
};
