import { unstable_cache } from "next/cache";
import { db, users } from "@/lib/db";
import { usernameEqualsIgnoreCase } from "@/lib/db/usernameLookup";
import type {
  ProfileSocialLink,
  ProfileSocialProvider,
} from "@/components/profile/types";

const GITHUB_API_BASE = "https://api.github.com";
const FETCH_TIMEOUT_MS = 5000;

const KNOWN_PROVIDERS: Record<
  string,
  Exclude<ProfileSocialProvider, "website">
> = {
  twitter: "twitter",
  linkedin: "linkedin",
  instagram: "instagram",
  facebook: "facebook",
  mastodon: "mastodon",
  bluesky: "bluesky",
  youtube: "youtube",
  twitch: "twitch",
  reddit: "reddit",
  npm: "npm",
};

function buildHeaders(): Record<string, string> {
  const headers: Record<string, string> = {
    Accept: "application/vnd.github+json",
    "User-Agent": "tokens-ci",
  };

  // Authenticating as the OAuth app lifts the unauthenticated 60 req/hour
  // rate limit to 5000 req/hour. Public data only — no user token involved.
  const clientId = process.env.GITHUB_CLIENT_ID;
  const clientSecret = process.env.GITHUB_CLIENT_SECRET;
  if (clientId && clientSecret) {
    const credentials = Buffer.from(`${clientId}:${clientSecret}`).toString(
      "base64",
    );
    headers.Authorization = `Basic ${credentials}`;
  }

  return headers;
}

function normalizeWebsiteUrl(blog: unknown): string | null {
  if (typeof blog !== "string") return null;
  const trimmed = blog.trim();
  if (!trimmed) return null;

  // GitHub stores the website field as free text, often without a scheme.
  const candidate = /^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(trimmed)
    ? trimmed
    : `https://${trimmed}`;

  try {
    const url = new URL(candidate);
    if (url.protocol !== "https:" && url.protocol !== "http:") return null;
    return url.toString();
  } catch {
    return null;
  }
}

function isHttpUrl(value: string): boolean {
  try {
    const url = new URL(value);
    return url.protocol === "https:" || url.protocol === "http:";
  } catch {
    return false;
  }
}

interface GitHubSocialFetch {
  links: ProfileSocialLink[];
  /** Both GitHub requests fulfilled with 2xx; false means the data is partial. */
  complete: boolean;
}

async function fetchGitHubSocialLinks(
  username: string,
): Promise<GitHubSocialFetch> {

  const headers = buildHeaders();
  const encoded = encodeURIComponent(username);

  const [profileResult, socialResult] = await Promise.allSettled([
    fetch(`${GITHUB_API_BASE}/users/${encoded}`, {
      headers,
      cache: "no-store",
      signal: AbortSignal.timeout(FETCH_TIMEOUT_MS),
    }),
    fetch(`${GITHUB_API_BASE}/users/${encoded}/social_accounts`, {
      headers,
      cache: "no-store",
      signal: AbortSignal.timeout(FETCH_TIMEOUT_MS),
    }),
  ]);

  const links: ProfileSocialLink[] = [];

  if (profileResult.status === "fulfilled" && profileResult.value.ok) {
    const profile: unknown = await profileResult.value.json();
    const blog =
      profile && typeof profile === "object" && "blog" in profile
        ? (profile as { blog: unknown }).blog
        : null;
    const website = normalizeWebsiteUrl(blog);
    if (website) {
      links.push({ provider: "website", url: website });
    }
  }

  if (socialResult.status === "fulfilled" && socialResult.value.ok) {
    const accounts: unknown = await socialResult.value.json();
    if (Array.isArray(accounts)) {
      for (const account of accounts) {
        const providerName =
          account && typeof account === "object" && "provider" in account
            ? (account as { provider: unknown }).provider
            : null;
        const url =
          account && typeof account === "object" && "url" in account
            ? (account as { url: unknown }).url
            : null;
        const provider =
          typeof providerName === "string"
            ? KNOWN_PROVIDERS[providerName]
            : undefined;
        if (
          provider &&
          typeof url === "string" &&
          isHttpUrl(url) &&
          !links.some((link) => link.url === url)
        ) {
          links.push({ provider, url });
        }
      }
    }
  }

  // A 404 means the username has no GitHub profile — a successfully refreshed
  // *empty* snapshot, not a failure. Only transport errors and non-404 non-OK
  // responses (5xx, rate limits) make the fetch incomplete.
  const usable = (r: PromiseSettledResult<Response>) =>
    r.status === "fulfilled" && (r.value.ok || r.value.status === 404);
  const complete = usable(profileResult) && usable(socialResult);

  return { links, complete };


}

async function persistSocialLinks(
  username: string,
  links: ProfileSocialLink[],
): Promise<void> {
  await db
    .update(users)
    .set({ socialLinks: links, socialLinksSyncedAt: new Date() })
    .where(usernameEqualsIgnoreCase(username));
}

/**
 * Strict variant for the maintenance cron: throws when either GitHub request
 * failed or the snapshot write failed, so a scheduled run can tell a real
 * refresh from silently stale data.
 */
export async function syncGitHubSocialLinksStrict(
  username: string,
): Promise<ProfileSocialLink[]> {
  const { links, complete } = await fetchGitHubSocialLinks(username);
  if (!complete) {
    throw new Error(`GitHub social fetch incomplete for ${username}`);
  }
  await persistSocialLinks(username, links);
  return links;
}


/**
 * Fetch the user's current social links from GitHub and persist the snapshot
 * on their users row (rendered by the profile page). Never throws.
 */
export async function syncGitHubSocialLinks(
  username: string,
): Promise<ProfileSocialLink[]> {
  try {
    const { links } = await fetchGitHubSocialLinks(username);

    await persistSocialLinks(username, links);
    return links;
  } catch {
    return [];
  }
}

/**
 * Social links from the user's public GitHub profile: the website field plus
 * recognized entries from the social accounts API. Failures resolve to an
 * empty list — this is a profile enrichment, never a render blocker.
 * Refreshes also persist the snapshot to the users row (at most once per
 * cache window per user).
 */
export function getGitHubSocialLinks(
  username: string,
): Promise<ProfileSocialLink[]> {
  const cacheKey = username.toLowerCase();

  return unstable_cache(
    () => syncGitHubSocialLinks(username),
    [`github-socials:${cacheKey}`],
    {
      tags: ["github-socials", `github-socials:${cacheKey}`],
      revalidate: 3600,
    },
  )();
}
