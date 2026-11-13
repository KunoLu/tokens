"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useTheme } from "next-themes";
import { LogOutIcon, MenuIcon, MoonIcon, SettingsIcon, SunIcon, UserIcon } from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button, buttonVariants } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
import { CONTAINER } from "@/components/layout/Container";
import { useLocale, useT } from "@/components/layout/LocaleProvider";
import { LanguageSwitcher } from "@/components/layout/LanguageSwitcher";
import { localizeHref, stripLocale } from "@/lib/i18n/config";
import { avatarUrlFor } from "@/lib/avatar";

interface User {
  id: string;
  username: string;
  displayName: string | null;
  avatarUrl: string | null;
}

/**
 * `label` is a catalogue key rather than the text. The list is module scope
 * because its order and matching do not change per render, and only the label
 * does — so the translation happens where each entry is rendered, not here.
 */
const NAV_LINKS = [
  { href: "/leaderboard", label: "nav.leaderboard", authOnly: false, match: (p: string) => p === "/leaderboard" },
  { href: "/teamboard", label: "nav.teamboard", authOnly: false, match: (p: string) => p === "/teamboard" },
  { href: "/teams", label: "nav.teams", authOnly: true, match: (p: string) => p === "/teams" },
  { href: "/docs", label: "nav.docs", authOnly: false, match: (p: string) => p.startsWith("/docs") },
  { href: "/profile", label: "nav.profile", authOnly: true, match: (p: string) => p === "/profile" || p.startsWith("/u/") },
] as const;

// returnTo deliberately skips the auth pages: pointing it there would send the
// user "back" to the sign-in form after a successful sign-in. Those clicks land
// on the default /leaderboard instead.
const AUTH_PAGES = ["/login", "/register", "/forgot-password", "/reset-password", "/verify-email"];

/**
 * The blue tile, so the corner of the page and the browser tab show the same
 * thing. Colours are literal rather than themed: this is the brand tile, and
 * it has to read identically against a light header, a dark header and the
 * favicon beside it.
 */
function TokensMark() {
  return (
    <svg viewBox="0 0 64 64" width={22} height={22} aria-hidden="true" className="shrink-0">
      <rect width={64} height={64} rx={14} fill="#2F6FDB" />
      <g stroke="#FFFFFF" strokeWidth={5} strokeLinecap="round" fill="none">
        <path d="M14 15h36" />
        <path d="M32 15v34" />
        <path d="M22 27h20" opacity={0.75} />
        <path d="M24.5 37h15" opacity={0.5} />
        <path d="M27 47h10" opacity={0.3} />
      </g>
    </svg>
  );
}

/**
 * Theme toggle. Rendering is deferred until mount because the resolved theme
 * is unknown during SSR, and swapping the icon on hydration would otherwise
 * flash the wrong one.
 */
function ThemeToggle() {
  const t = useT();
  const { resolvedTheme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setMounted(true);
  }, []);

  const isDark = resolvedTheme === "dark";

  if (!mounted) {
    return <Skeleton className="size-8 rounded-md" />;
  }

  return (
    <Button
      variant="ghost"
      size="icon"
      className="size-8"
      aria-label={isDark ? t("nav.switchToLight") : t("nav.switchToDark")}
      onClick={() => setTheme(isDark ? "light" : "dark")}
    >
      {isDark ? <MoonIcon /> : <SunIcon />}
    </Button>
  );
}

function UserMenu({ user, onSignOut }: { user: User; onSignOut: () => void }) {
  const t = useT();
  const locale = useLocale();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={
          <button
            aria-label={t("nav.accountMenu", { username: user.username })}
            className="rounded-full ring-offset-background transition-opacity hover:opacity-85 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          />
        }
      >
        <Avatar className="size-8">
          <AvatarImage src={avatarUrlFor(user)} alt="" />
          <AvatarFallback className="text-[10px]">
            {user.username.slice(0, 2).toUpperCase()}
          </AvatarFallback>
        </Avatar>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-52">
        <div className="flex flex-col gap-0.5 px-2 py-1.5">
          <span className="truncate text-sm font-medium">
            {user.displayName || user.username}
          </span>
          <span className="truncate text-xs text-muted-foreground">@{user.username}</span>
        </div>
        <DropdownMenuSeparator />
        <DropdownMenuGroup>
          <DropdownMenuItem render={<Link href={localizeHref(`/u/${user.username}`, locale)} />}>
            <UserIcon />
            {t("nav.yourProfile")}
          </DropdownMenuItem>
          <DropdownMenuItem render={<Link href={localizeHref("/settings", locale)} />}>
            <SettingsIcon />
            {t("nav.settings")}
          </DropdownMenuItem>
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuGroup>
          <DropdownMenuItem variant="destructive" onClick={onSignOut}>
            <LogOutIcon />
            {t("nav.signOut")}
          </DropdownMenuItem>
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export function Navigation() {
  const pathname = usePathname();
  const t = useT();
  const locale = useLocale();
  // The prefix is not part of what a nav entry matches on: `/zh-CN/leaderboard`
  // has to light up "Leaderboard" exactly like `/leaderboard` does.
  const currentPath = stripLocale(pathname || "/").path;
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  // A failed session request is not a sign-out. Holding the skeleton keeps the
  // header from flipping an authenticated user to a "Sign in" button.
  const [sessionFailed, setSessionFailed] = useState(false);

  // Retry rather than latch. Holding the skeleton on the first failure was
  // meant to avoid flipping an authenticated user to "Sign in", but with no
  // retry it never came back: one bad response left the header showing an
  // avatar placeholder forever, with the auth-only links hidden and no way to
  // sign in at all — including for signed-out visitors. A deploy takes this
  // service through roughly a minute of 502s, so that was a routine state, not
  // an edge case. After the retries are spent, fall through to the signed-out
  // header: a wrongly-shown "Sign in" costs one page view, while a
  // permanent skeleton costs a manual reload the user has no reason to guess.
  useEffect(() => {
    let cancelled = false;
    const delays = [800, 2000, 5000];

    const attempt = async (tries = 0): Promise<void> => {
      try {
        const res = await fetch("/api/auth/session");
        if (!res.ok) throw new Error("Failed to load session");
        const data = await res.json();
        if (cancelled) return;
        setUser(data.user || null);
        setSessionFailed(false);
        setIsLoading(false);
      } catch {
        if (cancelled) return;
        if (tries < delays.length) {
          setSessionFailed(true);
          setTimeout(() => {
            if (!cancelled) void attempt(tries + 1);
          }, delays[tries]);
          return;
        }
        setSessionFailed(false);
        setUser(null);
        setIsLoading(false);
      }
    };

    void attempt();
    return () => {
      cancelled = true;
    };
  }, []);

  const signOut = async () => {
    await fetch("/api/auth/logout", { method: "POST" });
    setUser(null);
    // Back to the board in the language being read, not back to English.
    window.location.href = localizeHref("/leaderboard", locale);
  };

  // Sign-in comes back to where it was clicked rather than dropping everyone on
  // /leaderboard, which is what the route falls back to when returnTo is absent.
  // Pathname only: the query string would need useSearchParams, which forces a
  // Suspense boundary around the whole header.
  const returnTo =
    pathname.startsWith("/") && !AUTH_PAGES.includes(currentPath)
      ? pathname
      : localizeHref("/leaderboard", locale);

  const links = NAV_LINKS.filter((l) => !l.authOnly || user);
  const hrefFor = (link: (typeof NAV_LINKS)[number]) =>
    localizeHref(
      link.href === "/profile" && user ? `/u/${user.username}` : link.href,
      locale,
    );

  return (
    <header className="sticky top-0 z-50 border-b bg-background/85 backdrop-blur-xl">
      <nav
        aria-label={t("nav.main")}
        className={cn(CONTAINER, "flex h-14 items-center gap-2")}
      >
        <Link
          href={localizeHref("/leaderboard", locale)}
          className="flex shrink-0 items-center gap-2"
          aria-label={t("nav.home")}
        >
          {/* The mark paints with currentColor, so one file covers both themes. */}
          <TokensMark />
          <span className="text-[15px] font-semibold tracking-tight">Tokens</span>
        </Link>

        <div className="ml-3 hidden items-center gap-0.5 sm:flex">
          {links.map((link) => {
            const active = link.match(currentPath);
            return (
              <Link
                key={link.href}
                href={hrefFor(link)}
                aria-current={active ? "page" : undefined}
                className={cn(
                  "rounded-md px-2.5 py-1.5 text-sm transition-colors",
                  active
                    ? "font-medium text-foreground"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                {t(link.label)}
              </Link>
            );
          })}
        </div>

        <div className="ml-auto flex items-center gap-1.5">
          <LanguageSwitcher />

          <ThemeToggle />

          {isLoading || sessionFailed ? (
            <Skeleton className="size-8 rounded-full" />
          ) : user ? (
            <UserMenu user={user} onSignOut={signOut} />
          ) : (
            <a
              href={`/login?returnTo=${encodeURIComponent(returnTo)}`}
              className={cn(
                buttonVariants({ size: "sm" }),
                "h-8 bg-[#7C3AED] text-white hover:bg-[#7C3AED]/90"
              )}
            >
              {t("nav.signIn")}
            </a>
          )}

          {/* Mobile navigation lives behind a menu; the links do not fit. */}
          <DropdownMenu>
            <DropdownMenuTrigger
              render={
                <Button
                  variant="ghost"
                  size="icon"
                  className="size-8 sm:hidden"
                  aria-label={t("nav.openMenu")}
                />
              }
            >
              <MenuIcon />
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-48">
              <DropdownMenuGroup>
                {links.map((link) => (
                  <DropdownMenuItem key={link.href} render={<Link href={hrefFor(link)} />}>
                    {t(link.label)}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </nav>
    </header>
  );
}
