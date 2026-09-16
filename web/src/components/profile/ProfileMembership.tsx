"use client";

import { useEffect, useState } from "react";
import { useRouter } from "nextjs-toploader/app";
import { toast } from "react-toastify";
import { Button } from "@/components/ui/button";
import { teamApi } from "@/components/teams/api";
import { useI18n } from "@/lib/i18n";
import type { ProfileMembership } from "@/lib/teams/profileMembership";

/**
 * Team/Group block on the public profile. Names render for everyone from
 * server data; the leave buttons appear only after the client session check
 * proves the viewer owns this profile — /u/* HTML is shared across visitors
 * (cached 60s), so ownership can never be baked into it.
 */
export function ProfileMembership({
  username,
  membership,
}: {
  username: string;
  membership: ProfileMembership;
}) {
  const { t } = useI18n();
  const router = useRouter();
  const [sessionUsername, setSessionUsername] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const isOwner =
    sessionUsername !== null &&
    sessionUsername.toLowerCase() === username.toLowerCase();

  useEffect(() => {
    let cancelled = false;
    fetch("/api/auth/session")
      .then((res) => (res.ok ? res.json() : null))
      .then((data: { user?: { username?: unknown } | null } | null) => {
        if (cancelled) return;
        const next = data?.user?.username;
        setSessionUsername(typeof next === "string" ? next : null);
      })
      .catch(() => {
        if (!cancelled) setSessionUsername(null);
      });
    return () => {
      cancelled = true;
    };
  }, [username]);

  const leave = async (path: string) => {
    setBusy(true);
    const error = await teamApi(path, { method: "DELETE" });
    setBusy(false);
    if (error) {
      // TeamError messages are user-facing; the 409 admin guard copy
      // ("Transfer admin or disband the team before leaving") arrives as-is.
      toast.error(error);
      return;
    }
    router.refresh();
  };

  const group = membership.group;

  return (
    <section
      aria-label={t("profile.membership")}
      className="mt-5 flex flex-wrap items-center gap-x-4 gap-y-2 text-sm"
    >
      <span>
        <span className="text-muted-foreground">{t("profile.team")}: </span>
        {membership.team.name}
      </span>
      {group && (
        <span>
          <span className="text-muted-foreground">{t("profile.group")}: </span>
          {group.name}
        </span>
      )}
      {isOwner && (
        <span className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={busy}
            onClick={() =>
              void leave(`/api/teams/${membership.team.id}/members/me`)
            }
          >
            {t("teams.leave")}
          </Button>
          {group && (
            <Button
              variant="outline"
              size="sm"
              disabled={busy}
              onClick={() =>
                void leave(
                  `/api/teams/${membership.team.id}/groups/${group.id}/members/me`
                )
              }
            >
              {t("profile.leaveGroup")}
            </Button>
          )}
        </span>
      )}
    </section>
  );
}
