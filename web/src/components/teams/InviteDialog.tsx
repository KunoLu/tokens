"use client";

import { useEffect, useRef, useState } from "react";
import { toast } from "react-toastify";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { avatarUrlFor } from "@/lib/avatar";
import { localizeServerError, localizeServerErrorList, useI18n } from "@/lib/i18n";

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

interface SearchUser {
  username: string;
  displayName: string | null;
  avatarUrl: string | null;
}

type Selection =
  | { kind: "user"; username: string; displayName: string | null; avatarUrl: string | null }
  | { kind: "email"; email: string };

/**
 * Invite combobox: type to filter accounts via GET /api/users/search, tick
 * options to build a multi-selection with a live count, and — when the input
 * is a full email that matches no account — invite that address directly.
 * Options only ever show username + displayName; emails are never echoed.
 */
export function InviteDialog({
  teamId,
  memberUsernames,
  groups,
  open,
  onOpenChange,
}: {
  teamId: string;
  /** Lowercase usernames already in the team; filtered out of the options. */
  memberUsernames: string[];
  /** Active groups the invitee can be auto-assigned to on accept. */
  groups: { id: string; name: string }[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const { t } = useI18n();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchUser[]>([]);
  const [selected, setSelected] = useState<Map<string, Selection>>(new Map());
  const [busy, setBusy] = useState(false);
  const [groupId, setGroupId] = useState<string | null>(null);
  const requestSeq = useRef(0);

  const members = useRef(new Set(memberUsernames.map((u) => u.toLowerCase())));
  members.current = new Set(memberUsernames.map((u) => u.toLowerCase()));

  useEffect(() => {
    if (!open) {
      requestSeq.current += 1;
      return;
    }
    const q = query.trim();
    if (!q) {
      requestSeq.current += 1;
      setResults([]);
      return;
    }
    const seq = ++requestSeq.current;
    const timer = setTimeout(async () => {
      try {
        const res = await fetch(`/api/users/search?q=${encodeURIComponent(q)}`);
        if (seq !== requestSeq.current) return;
        if (!res.ok) {
          toast.error(t("invite.searchFailed"));
          setResults([]);
          return;
        }
        const data: { users?: SearchUser[] } = await res.json();
        setResults(
          (data.users ?? []).filter(
            (u) => !members.current.has(u.username.toLowerCase())
          )
        );
      } catch {
        if (seq === requestSeq.current) setResults([]);
      }
    }, 250);
    return () => clearTimeout(timer);
  }, [query, open, t]);

  const reset = () => {
    setQuery("");
    setResults([]);
    setSelected(new Map());
    setGroupId(null);
  };

  const toggle = (key: string, selection: Selection) => {
    setSelected((prev) => {
      const next = new Map(prev);
      if (next.has(key)) next.delete(key);
      else next.set(key, selection);
      return next;
    });
  };

  const trimmed = query.trim();
  const showEmailRow =
    EMAIL_PATTERN.test(trimmed) &&
    results.length === 0 &&
    !selected.has(`e:${trimmed.toLowerCase()}`);

  const send = async () => {
    setBusy(true);
    try {
      const items = [...selected.values()].map((s) => ({
        ...(s.kind === "user" ? { username: s.username } : { email: s.email }),
        ...(groupId ? { groupId } : {}),
      }));
      const res = await fetch(`/api/teams/${teamId}/members`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ items }),
      });
      const data: {
        created?: { id: string; skipped?: boolean }[];
        details?: string[];
        error?: string;
      } = await res.json().catch(() => ({}));
      if (!res.ok) {
        toast.error(
          data.details && data.details.length > 0
            ? localizeServerErrorList(t, data.details).join("; ")
            : data.error
              ? localizeServerError(t, data.error)
              : `HTTP ${res.status}`
        );
        return;
      }
      const sent = (data.created ?? []).filter((row) => row.id).length;
      if (data.details && data.details.length > 0) {
        toast.error(localizeServerErrorList(t, data.details).join("; "));
      }
      if (sent > 0) {
        toast.success(t("invite.sent", { n: sent }));
        reset();
        onOpenChange(false);
      }
    } catch {
      toast.error(t("invite.sendFailed"));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (!next) reset();
        onOpenChange(next);
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("invite.title")}</DialogTitle>
          <DialogDescription>{t("invite.desc")}</DialogDescription>
        </DialogHeader>

        <Input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={t("invite.searchPh")}
          autoComplete="off"
          aria-label={t("invite.searchPh")}
        />

        <div className="flex max-h-56 flex-col overflow-y-auto rounded-lg border border-border">
          {results.map((user) => {
            const key = `u:${user.username.toLowerCase()}`;
            const checked = selected.has(key);
            return (
              <button
                key={key}
                type="button"
                className="flex items-center gap-2.5 px-2.5 py-2 text-left text-sm hover:bg-muted"
                onClick={() =>
                  toggle(key, {
                    kind: "user",
                    username: user.username,
                    displayName: user.displayName,
                    avatarUrl: user.avatarUrl,
                  })
                }
              >
                <Checkbox checked={checked} tabIndex={-1} render={<span className="pointer-events-none" />} />
                <Avatar className="size-7">
                  <AvatarImage
                    src={avatarUrlFor({
                      username: user.username,
                      avatarUrl: user.avatarUrl,
                    })}
                    alt=""
                  />
                  <AvatarFallback>
                    {user.username.slice(0, 2).toUpperCase()}
                  </AvatarFallback>
                </Avatar>
                <span className="min-w-0 flex-1">
                  <span className="block truncate font-medium">
                    {user.displayName ?? user.username}
                  </span>
                  <span className="block truncate text-xs text-muted-foreground">
                    @{user.username}
                  </span>
                </span>
              </button>
            );
          })}
          {showEmailRow && (
            <button
              type="button"
              className="flex items-center gap-2.5 px-2.5 py-2 text-left text-sm hover:bg-muted"
              onClick={() =>
                toggle(`e:${trimmed.toLowerCase()}`, {
                  kind: "email",
                  email: trimmed.toLowerCase(),
                })
              }
            >
              <Checkbox checked={false} tabIndex={-1} render={<span className="pointer-events-none" />} />
              <span className="flex-1">{t("invite.inviteEmail", { email: trimmed })}</span>
            </button>
          )}
          {trimmed && !showEmailRow && results.length === 0 && (
            <p className="px-2.5 py-2.5 text-xs text-muted-foreground">
              {t("invite.noMatch")}
            </p>
          )}
          {!trimmed && selected.size === 0 && (
            <p className="px-2.5 py-2.5 text-xs text-muted-foreground">
              {t("invite.searchPh")}
            </p>
          )}
          {[...selected.entries()]
            .filter(([key]) => {
              // Selected rows stay visible even when they no longer match the
              // query, so a tick is never lost behind the filter.
              if (key.startsWith("e:")) return true;
              return !results.some(
                (u) => `u:${u.username.toLowerCase()}` === key
              );
            })
            .map(([key, s]) => (
              <button
                key={key}
                type="button"
                className="flex items-center gap-2.5 px-2.5 py-2 text-left text-sm hover:bg-muted"
                onClick={() => toggle(key, s)}
              >
                <Checkbox checked tabIndex={-1} render={<span className="pointer-events-none" />} />
                <span className="min-w-0 flex-1 truncate">
                  {s.kind === "user"
                    ? `${s.displayName ?? s.username} (@${s.username})`
                    : t("invite.inviteEmail", { email: s.email })}
                </span>
              </button>
            ))}
        </div>

        {groups.length > 0 && (
          <div className="flex flex-col gap-1.5">
            <Label>{t("invite.autoAssign")}</Label>
            <Select
              value={groupId ?? "none"}
              onValueChange={(value) =>
                setGroupId(value === "none" ? null : value)
              }
            >
              <SelectTrigger aria-label={t("invite.autoAssign")}>
                <SelectValue>
                  {(value) =>
                    value === "none"
                      ? t("invite.noGroup")
                      : (groups.find((g) => g.id === value)?.name ??
                        t("invite.noGroup"))
                  }
                </SelectValue>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">{t("invite.noGroup")}</SelectItem>
                {groups.map((g) => (
                  <SelectItem key={g.id} value={g.id}>
                    {g.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}

        <p className="text-xs text-muted-foreground tabular">
          {t("invite.count", { n: selected.size })}
        </p>

        <DialogFooter>
          <DialogClose render={<Button variant="outline" disabled={busy} />}>
            {t("dlg.cancel")}
          </DialogClose>
          <Button
            disabled={busy || selected.size === 0}
            onClick={() => void send()}
          >
            {t("invite.send")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
