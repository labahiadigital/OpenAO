import type { Handle } from "@sveltejs/kit";

export const handle: Handle = async ({ event, resolve }) => {
  const sessionCookie = event.cookies.get("session");
  if (sessionCookie) {
    try {
      event.locals.session = JSON.parse(atob(sessionCookie));
    } catch {
      event.locals.session = null;
    }
  } else {
    event.locals.session = null;
  }

  return resolve(event);
};
