"use client";

import { useEffect, useState, type ReactNode } from "react";
import { usePathname, useSearchParams } from "next/navigation";
import type { ProfileDevice } from "@/components/profile";
import ProfilePageClient, { type ProfileData } from "./ProfilePageClient";

type OwnerState =
  | { kind: "hidden" }
  | { kind: "owner"; data: ProfileData; devices: ProfileDevice[] };

function usernameFromPath(pathname: string | null): string | null {
  const match = pathname?.match(/^\/u\/([^/]+)\/?$/);
  if (!match) return null;
  try {
    return decodeURIComponent(match[1]);
  } catch {
    return null;
  }
}

/**
 * The cached profile page is identical for a missing user and a private one.
 * After that HTML loads, the signed-in owner replaces it with their own profile.
 */
export default function PrivateProfileOwnerView({ fallback }: { fallback: ReactNode }) {
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const username = usernameFromPath(pathname);
  const period = searchParams.get("period");
  const [state, setState] = useState<OwnerState>({ kind: "hidden" });

  useEffect(() => {
    if (!username) return;
    let cancelled = false;
    const target = username;

    async function load() {
      try {
        const sessionResponse = await fetch("/api/auth/session");
        if (!sessionResponse.ok) return;
        const session = (await sessionResponse.json()) as {
          user?: { username?: string } | null;
        };
        if (session.user?.username?.toLowerCase() !== target.toLowerCase()) return;

        const query = period ? `?period=${encodeURIComponent(period)}` : "";
        const [profileResponse, devicesResponse] = await Promise.all([
          fetch(`/api/users/${encodeURIComponent(target)}${query}`),
          fetch(`/api/users/${encodeURIComponent(target)}/devices`),
        ]);
        if (!profileResponse.ok) return;
        const data = (await profileResponse.json()) as ProfileData;
        const devicesPayload = devicesResponse.ok
          ? ((await devicesResponse.json()) as { devices?: ProfileDevice[] })
          : { devices: [] };
        if (cancelled || data.user?.username?.toLowerCase() !== target.toLowerCase()) return;
        setState({
          kind: "owner",
          data,
          devices: Array.isArray(devicesPayload.devices) ? devicesPayload.devices : [],
        });
      } catch {
        // Keep the shared not-found page.
      }
    }

    void load();
    return () => {
      cancelled = true;
    };
  }, [period, username]);

  if (state.kind === "owner" && username) {
    return (
      <ProfilePageClient
        initialData={state.data}
        initialDevices={state.devices}
        username={username}
      />
    );
  }

  return fallback;
}
