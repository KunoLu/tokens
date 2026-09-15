"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { CONTAINER } from "@/components/layout/Container";
import { PageHeader } from "@/components/layout/PageHeader";
import { cn } from "@/lib/utils";
import { useI18n } from "@/lib/i18n";

type State = "verifying" | "success" | "error";

export default function VerifyEmailClient({ token }: { token: string }) {
  const { t } = useI18n();
  const [state, setState] = useState<State>(token ? "verifying" : "error");
  const [message, setMessage] = useState(
    token ? "" : t("auth.verify.missingToken")
  );
  const [signedIn, setSignedIn] = useState(false);
  const [resending, setResending] = useState(false);
  const [resent, setResent] = useState(false);
  // The effect must run once; React StrictMode double-invokes effects in dev,
  // and the second run would consume an already-spent token.
  const started = useRef(false);

  useEffect(() => {
    void fetch("/api/auth/session")
      .then((response) => response.json())
      .then((data: { user?: unknown }) => setSignedIn(Boolean(data.user)))
      .catch(() => setSignedIn(false));
  }, []);

  useEffect(() => {
    if (!token || started.current) return;
    started.current = true;

    (async () => {
      try {
        const response = await fetch("/api/auth/verify-email", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ token }),
        });
        if (!response.ok) {
          const data = await response.json().catch(() => ({}));
          setState("error");
          setMessage(data.error || t("auth.verify.failed"));
          return;
        }
        setState("success");
      } catch {
        setState("error");
        setMessage(t("auth.verify.networkOpen"));
      }
    })();
  }, [token, t]);

  const resend = async () => {
    setResending(true);
    setResent(false);
    try {
      const response = await fetch("/api/auth/resend-verification", {
        method: "POST",
      });
      if (response.status === 401) {
        setMessage(t("auth.verify.signInResend"));
        setSignedIn(false);
        return;
      }
      if (!response.ok) {
        const data = await response.json().catch(() => ({}));
        setMessage(data.error || t("auth.verify.couldNotResend"));
        return;
      }
      setResent(true);
    } catch {
      setMessage(t("auth.network"));
    } finally {
      setResending(false);
    }
  };

  return (
    <main id="main-content" className={cn(CONTAINER, "max-w-[460px] pb-24 pt-10 sm:pt-14")}>
      <PageHeader
        title={t("auth.verify.title")}
        description={t("auth.verify.desc")}
      />

      {state === "verifying" && (
        <p className="text-sm text-muted-foreground">{t("auth.verify.confirming")}</p>
      )}

      {state === "success" && (
        <div role="status" className="rounded-lg border border-border bg-card px-4 py-3 text-sm">
          {t("auth.verify.success")}{" "}
          <Link href="/leaderboard" className="text-primary underline-offset-4 hover:underline">
            {t("auth.verify.back")}
          </Link>
        </div>
      )}

      {state === "error" && (
        <div className="flex flex-col gap-4">
          <div
            role="alert"
            className="rounded-lg border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm text-destructive"
          >
            {message}
          </div>
          {signedIn ? (
            <div className="flex flex-col gap-2">
              <Button type="button" onClick={resend} disabled={resending} className="h-9">
                {resending ? t("auth.sending") : t("auth.verify.resend")}
              </Button>
              {resent && (
                <p role="status" className="text-sm text-muted-foreground">
                  {t("auth.verify.resent")}
                </p>
              )}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">
              {t("auth.verify.signInThen")}{" "}
              <Link href="/login?returnTo=/verify-email" className="text-primary underline-offset-4 hover:underline">
                {t("nav.signIn")}
              </Link>
            </p>
          )}
        </div>
      )}
    </main>
  );
}
