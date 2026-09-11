"use client";

import { useState } from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { CONTAINER } from "@/components/layout/Container";
import { PageHeader } from "@/components/layout/PageHeader";
import { cn } from "@/lib/utils";

export default function ResetPasswordForm({ token }: { token: string }) {
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [details, setDetails] = useState<string[]>([]);
  const [done, setDone] = useState(false);
  const [submitting, setSubmitting] = useState(false);

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
        setError(data.error || "Failed to reset password");
        setDetails(Array.isArray(data.details) ? data.details : []);
        return;
      }
      setDone(true);
    } catch {
      setError("Network error — please try again");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <main id="main-content" className={cn(CONTAINER, "max-w-[460px] pb-24 pt-10 sm:pt-14")}>
      <PageHeader title="Choose a new password" />

      {!token ? (
        <div
          role="alert"
          className="rounded-lg border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm text-destructive"
        >
          This reset link is missing its token. Request a fresh link from the{" "}
          <Link href="/forgot-password" className="underline underline-offset-4">
            forgot password
          </Link>{" "}
          page.
        </div>
      ) : done ? (
        <div role="status" className="rounded-lg border border-border bg-card px-4 py-3 text-sm">
          Password updated. Every existing sign-in was ended —{" "}
          <Link href="/login" className="text-primary underline-offset-4 hover:underline">
            sign in
          </Link>{" "}
          with the new password.
        </div>
      ) : (
        <form onSubmit={onSubmit} className="flex flex-col gap-4">
          <div className="flex flex-col gap-1.5">
            <label htmlFor="password" className="text-[13px] font-semibold">
              New password
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
              At least 8 characters, with an uppercase letter, a lowercase letter
              and a special character.
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
            {submitting ? "Updating…" : "Update password"}
          </Button>
        </form>
      )}
    </main>
  );
}
