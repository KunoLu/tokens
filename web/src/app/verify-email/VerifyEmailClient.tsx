"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { CONTAINER } from "@/components/layout/Container";
import { PageHeader } from "@/components/layout/PageHeader";
import { cn } from "@/lib/utils";

type State = "verifying" | "success" | "error";

export default function VerifyEmailClient({ token }: { token: string }) {
  const [state, setState] = useState<State>(token ? "verifying" : "error");
  const [message, setMessage] = useState(
    token ? "" : "This verification link is missing its token."
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
          setMessage(data.error || "Verification failed");
          return;
        }
        setState("success");
      } catch {
        setState("error");
        setMessage("Network error — please open the link again");
      }
    })();
  }, [token]);

  const resend = async () => {
    setResending(true);
    setResent(false);
    try {
      const response = await fetch("/api/auth/resend-verification", {
        method: "POST",
      });
      if (response.status === 401) {
        setMessage("Sign in to resend the verification email.");
        setSignedIn(false);
        return;
      }
      if (!response.ok) {
        const data = await response.json().catch(() => ({}));
        setMessage(data.error || "Could not resend the email");
        return;
      }
      setResent(true);
    } catch {
      setMessage("Network error — please try again");
    } finally {
      setResending(false);
    }
  };

  return (
    <main id="main-content" className={cn(CONTAINER, "max-w-[460px] pb-24 pt-10 sm:pt-14")}>
      <PageHeader
        title="Verify your email"
        description="Open the link we sent, or resend it if the message never arrived."
      />

      {state === "verifying" && (
        <p className="text-sm text-muted-foreground">Confirming your address…</p>
      )}

      {state === "success" && (
        <div role="status" className="rounded-lg border border-border bg-card px-4 py-3 text-sm">
          Your email is verified.{" "}
          <Link href="/leaderboard" className="text-primary underline-offset-4 hover:underline">
            Back to the leaderboard
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
                {resending ? "Sending…" : "Resend verification email"}
              </Button>
              {resent && (
                <p role="status" className="text-sm text-muted-foreground">
                  If this account still needs verification, a new link is on its way.
                </p>
              )}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">
              Sign in, then come back here to resend the verification email.{" "}
              <Link href="/login?returnTo=/verify-email" className="text-primary underline-offset-4 hover:underline">
                Sign in
              </Link>
            </p>
          )}
        </div>
      )}
    </main>
  );
}
