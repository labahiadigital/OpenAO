import { proxyToApi } from "$lib/server/api-proxy";
import type { RequestHandler } from "./$types";

const handler: RequestHandler = async (event) => {
  const path = event.params.path;
  return proxyToApi(event, `/market/${path}`);
};

export const GET = handler;
export const POST = handler;
