import { getApiBaseUrl } from "$lib/api-base-url";
import type { RequestEvent } from "@sveltejs/kit";

export async function proxyToApi(
  event: RequestEvent,
  path: string,
): Promise<Response> {
  const apiBase = getApiBaseUrl();
  const url = `${apiBase}${path}`;

  const headers = new Headers();
  const cookie = event.request.headers.get("cookie");
  if (cookie) headers.set("cookie", cookie);
  const contentType = event.request.headers.get("content-type");
  if (contentType) headers.set("content-type", contentType);
  const authorization = event.request.headers.get("authorization");
  if (authorization) headers.set("authorization", authorization);
  headers.set("x-forwarded-for", event.getClientAddress());

  const init: RequestInit = {
    method: event.request.method,
    headers,
  };

  if (event.request.method !== "GET" && event.request.method !== "HEAD") {
    init.body = await event.request.text();
  }

  try {
    const response = await fetch(url, init);

    const responseHeaders = new Headers();
    for (const [key, value] of response.headers) {
      if (key.toLowerCase() === "set-cookie") {
        responseHeaders.append(key, value);
      } else if (key.toLowerCase() !== "transfer-encoding") {
        responseHeaders.set(key, value);
      }
    }

    return new Response(response.body, {
      status: response.status,
      statusText: response.statusText,
      headers: responseHeaders,
    });
  } catch {
    return new Response(
      JSON.stringify({ error: "API backend no disponible" }),
      {
        status: 503,
        headers: { "content-type": "application/json" },
      },
    );
  }
}
