"use client";

import { useState } from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { CONTAINER } from "@/components/layout/Container";
import { PageHeader } from "@/components/layout/PageHeader";
import { cn } from "@/lib/utils";
import { localizeServerError, useI18n } from "@/lib/i18n";

export default function ForgotPasswordForm({ returnTo }: { returnTo: string }) {
  const [email, setEmail] = useState("");
  const [sent, setSent] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const { t } = useI18n();

  const onSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      const response = await fetch("/api/auth/forgot-password", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email }),
      });
      if (!response.ok) {
        const data = await response.json().catch(() => ({}));
        setError(
          typeof data.error === "string"
            ? localizeServerError(t, data.error)
            : t("auth.somethingWrong")
        );
        return;
      }
      // The API answers 200 for every address, so this copy must too.
      setSent(true);
    } catch {
      setError(t("auth.network"));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <main id="main-content" className={cn(CONTAINER, "max-w-[460px] pb-24 pt-10 sm:pt-14")}>
      <PageHeader
        title={t("auth.forgot.title")}
        description={t("auth.forgot.desc")}
      />

      {sent ? (
        <div
          role="status"
          className="rounded-lg border border-border bg-card px-4 py-3 text-sm"
        >
          {t("auth.forgot.sent")}
        </div>
      ) : (
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

          {error && (
            <div
              role="alert"
              className="rounded-lg border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm text-destructive"
            >
              {error}
            </div>
          )}

          <Button type="submit" disabled={submitting} className="h-9">
            {submitting ? t("auth.sending") : t("auth.forgot.submit")}
          </Button>
        </form>
      )}

      <p className="mt-6 text-sm text-muted-foreground">
        {t("auth.forgot.remembered")}{" "}
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
