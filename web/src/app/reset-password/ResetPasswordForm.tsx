"use client";

import { useState } from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { CONTAINER } from "@/components/layout/Container";
import { PageHeader } from "@/components/layout/PageHeader";
import { cn } from "@/lib/utils";
import { useI18n } from "@/lib/i18n";

export default function ResetPasswordForm({ token }: { token: string }) {
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [details, setDetails] = useState<string[]>([]);
  const [done, setDone] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const { t } = useI18n();

  const onSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setSubmitting(true);
    setError(null);
    setDetails([]);
    try {
      const response = await fetch("/api/auth/reset-password", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ token, password }),
      });
      const data = await response.json().catch(() => ({}));
      if (!response.ok) {
        setError(data.error || t("auth.reset.failed"));
        setDetails(Array.isArray(data.details) ? data.details : []);
        return;
      }
      setDone(true);
    } catch {
      setError(t("auth.network"));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <main id="main-content" className={cn(CONTAINER, "max-w-[460px] pb-24 pt-10 sm:pt-14")}>
      <PageHeader title={t("auth.reset.title")} />

      {!token ? (
        <div
          role="alert"
          className="rounded-lg border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm text-destructive"
        >
          {t("auth.reset.missingToken")}{" "}
          <Link href="/forgot-password" className="underline underline-offset-4">
            {t("auth.reset.forgotPage")}
          </Link>{" "}
          {t("auth.reset.missingTokenEnd")}
        </div>
      ) : done ? (
        <div role="status" className="rounded-lg border border-border bg-card px-4 py-3 text-sm">
          {t("auth.reset.done")}{" "}
          <Link href="/login" className="text-primary underline-offset-4 hover:underline">
            {t("auth.reset.signIn")}
          </Link>{" "}
          {t("auth.reset.doneEnd")}
        </div>
      ) : (
        <form onSubmit={onSubmit} className="flex flex-col gap-4">
          <div className="flex flex-col gap-1.5">
            <label htmlFor="password" className="text-[13px] font-semibold">
              {t("auth.reset.newPassword")}
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
            {submitting ? t("auth.reset.submitting") : t("auth.reset.submit")}
          </Button>
        </form>
      )}
    </main>
  );
}
