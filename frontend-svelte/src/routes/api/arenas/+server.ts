import { proxyToApi } from "$lib/server/api-proxy";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async (event) => {
  return proxyToApi(event, "/arenas");
};
