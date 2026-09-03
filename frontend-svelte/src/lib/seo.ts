import { env } from "$env/dynamic/public";

const rawSiteUrl = env.PUBLIC_SITE_URL?.trim() || "https://aoweb.app";
const normalizedSiteUrl = rawSiteUrl.startsWith("http")
  ? rawSiteUrl
  : `https://${rawSiteUrl}`;

export const siteUrl = normalizedSiteUrl.replace(/\/+$/, "");
export const siteName = "AOWeb";
export const siteTitle = "AOWeb Beta";
export const siteDescription = "";
export const siteKeywords = [
  "AOWeb",
  "AO Web",
  "MMORPG web",
  "AOWeb beta",
  "changelog AOWeb",
  "roadmap AOWeb",
];

export function absoluteUrl(path = "/"): string {
  return new URL(path, `${siteUrl}/`).toString();
}
