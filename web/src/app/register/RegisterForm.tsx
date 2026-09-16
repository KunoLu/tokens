"use client";

import { useState } from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { CONTAINER } from "@/components/layout/Container";
import { PageHeader } from "@/components/layout/PageHeader";
import { cn } from "@/lib/utils";
import { localizeServerError, localizeServerErrorList, useI18n } from "@/lib/i18n";

export default function RegisterForm({ returnTo }: { returnTo: string }) {
  const [email, setEmail] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [details, setDetails] = useState<string[]>([]);
  const [submitting, setSubmitting] = useState(false);
  const { t } = useI18n();

  const onSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setSubmitting(true);
    setError(null);
    setDetails([]);
    try {
      const response = await fetch("/api/auth/register", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, username, password }),
      });
      const data = await response.json().catch(() => ({}));
      if (!response.ok) {
        setError(
          typeof data.error === "string"
            ? localizeServerError(t, data.error)
            : t("auth.register.failed")
        );
        setDetails(
          Array.isArray(data.details)
            ? localizeServerErrorList(
                t,
                data.details.filter((item: unknown): item is string => typeof item === "string")
              )
            : []
        );
        return;
      }
      // Full reload so the navigation picks up the fresh session cookie.
      window.location.href = returnTo;
    } catch {
      setError(t("auth.network"));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <main id="main-content" className={cn(CONTAINER, "max-w-[460px] pb-24 pt-10 sm:pt-14")}>
      <PageHeader
        title={t("auth.register.title")}
        description={t("auth.register.desc")}
      />

      <form onSubmit={onSubmit} className="flex flex-col gap-4">
        <div className="flex flex-col gap-1.5">
          <label htmlFor="email" className="text-[13px] font-semibold">
            {t("auth.email")}
          </label>
          <Input
            id="email"
            type="email"
            autoComplete="email"
            required
            value={email}
            onChange={(e) => setEmail(e.target.value)}
          />
        </div>

        <div className="flex flex-col gap-1.5">
          <label htmlFor="username" className="text-[13px] font-semibold">
            {t("auth.username")}
          </label>
          <Input
            id="username"
            type="text"
            autoComplete="username"
            required
            maxLength={39}
            value={username}
            onChange={(e) => setUsername(e.target.value)}
          />
          <p className="text-xs text-muted-foreground">
            {t("auth.register.usernameHint")}
          </p>
        </div>

        <div className="flex flex-col gap-1.5">
          <label htmlFor="password" className="text-[13px] font-semibold">
            {t("auth.password")}
          </label>
          <Input
            id="password"
            type="password"
            autoComplete="new-password"
            required
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
          <p className="text-xs text-muted-foreground">
            {t("auth.register.passwordHint")}
          </p>
        </div>

        {error && (
          <div
            role="alert"
            className="rounded-lg border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm text-destructive"
          >
            <p>{error}</p>
            {details.length > 0 && (
              <ul className="mt-1 list-inside list-disc">
                {details.map((detail) => (
                  <li key={detail}>{detail}</li>
                ))}
              </ul>
            )}
          </div>
        )}

        <Button type="submit" disabled={submitting} className="h-9">
          {submitting ? t("auth.register.submitting") : t("auth.register.submit")}
        </Button>
      </form>

      <p className="mt-6 text-sm text-muted-foreground">
        {t("auth.register.haveAccount")}{" "}
        <Link
          href={`/login?returnTo=${encodeURIComponent(returnTo)}`}
          className="text-primary underline-offset-4 hover:underline"
        >
          {t("nav.signIn")}
        </Link>
      </p>
    </main>
  );
}
