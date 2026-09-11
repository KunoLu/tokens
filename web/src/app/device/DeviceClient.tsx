"use client";

import { useCallback, useEffect, useRef, useState } from "react";

import { Panel } from "@/components/ui/primitives";

interface User {
  id: string;
  username: string;
  displayName: string | null;
  avatarUrl: string | null;
}

export default function DeviceClient() {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  // "Could not load the session" is not the same answer as "signed out":
  // treating a failed request as signed out sends an authenticated user back
  // through sign-in for no reason.
  const [sessionFailed, setSessionFailed] = useState(false);
  const [code, setCode] = useState("");
  const [status, setStatus] = useState<"idle" | "loading" | "success" | "error">("idle");
  const [error, setError] = useState("");
  const successHeadingRef = useRef<HTMLHeadingElement>(null);

  const loadSession = useCallback(() => {
    setIsLoading(true);
    setSessionFailed(false);
    fetch("/api/auth/session")
      .then((res) => {
        if (!res.ok) throw new Error("Failed to load session");
        return res.json();
      })
      .then((data) => {
        setUser(data.user ?? null);
        setIsLoading(false);
      })
      .catch(() => {
        setSessionFailed(true);
        setIsLoading(false);
      });
  }, []);

  useEffect(() => {
    loadSession();
  }, [loadSession]);

  // The form the user was operating is replaced wholesale on success, so focus
  // would otherwise be left on a button that no longer exists.
  useEffect(() => {
    if (status === "success") successHeadingRef.current?.focus();
  }, [status]);

  const handleCodeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    let value = e.target.value.toUpperCase().replace(/[^A-Z0-9]/g, "");
    if (value.length > 4) value = value.slice(0, 4) + "-" + value.slice(4, 8);
    setCode(value);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setStatus("loading");
    setError("");
    try {
      const response = await fetch("/api/auth/device/authorize", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ userCode: code }),
      });
      if (!response.ok) {
        const data = await response.json();
        throw new Error(data.error || "Invalid code");
      }
      setStatus("success");
    } catch (err) {
      setStatus("error");
      setError(err instanceof Error ? err.message : "Something went wrong");
    }
  };

  if (isLoading) {
    return (
      <main id="main-content" className="flex flex-1 items-center justify-center bg-background px-4">
        <div className="text-sm text-muted-foreground">Loading...</div>
      </main>
    );
  }

  return (
    <main id="main-content" className="mx-auto mt-10 w-full max-w-[460px] px-4">
      <Panel className="p-6">
        <div className="mb-6 text-center">
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-lg border border-border bg-muted">
            <svg
              className="h-6 w-6 text-primary"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
              />
            </svg>
          </div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground">Authorize CLI</h1>
          <p className="mt-1 text-sm text-muted-foreground">Connect your terminal to Tokens</p>
        </div>

        {sessionFailed ? (
          <div className="text-center" role="alert">
            <p className="mb-5 text-sm text-muted-foreground">
              We could not check whether you are signed in. You may still be —
              this is a connection problem, not a sign-out.
            </p>
            <button
              type="button"
              onClick={loadSession}
              className="inline-flex h-11 w-full items-center justify-center rounded-lg bg-primary px-4 text-sm font-semibold text-primary-foreground transition hover:opacity-90"
            >
              Try again
            </button>
          </div>
        ) : !user ? (
          <div className="text-center">
            <p className="mb-5 text-sm text-muted-foreground">Sign in to authorize the CLI.</p>
            <a
              href="/login?returnTo=/device"
              className="inline-flex h-11 w-full items-center justify-center rounded-lg bg-primary px-4 text-sm font-semibold text-primary-foreground transition hover:opacity-90"
            >
              Sign in
            </a>
          </div>
        ) : status === "success" ? (
          <div className="text-center" role="status" aria-live="polite">
            <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full border border-success/40 bg-success/10">
              <svg
                className="h-6 w-6 text-success"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                aria-hidden="true"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <h2
              ref={successHeadingRef}
              tabIndex={-1}
              className="mb-1 text-lg font-semibold tracking-tight text-foreground outline-none"
            >
              Device Authorized!
            </h2>
            <p className="text-sm text-muted-foreground">You can close this window and return to your terminal.</p>
          </div>
        ) : (
          <form onSubmit={handleSubmit}>
            <div className="mb-4">
              <label htmlFor="device-code" className="mb-2 block text-center text-sm text-muted-foreground">
                Enter the code shown in your terminal:
              </label>
              <input
                id="device-code"
                type="text"
                value={code}
                onChange={handleCodeChange}
                placeholder="XXXX-XXXX"
                maxLength={9}
                autoFocus
                aria-label="Device code"
                className="w-full rounded-lg border border-border bg-muted px-4 py-4 text-center font-mono text-3xl font-bold tracking-[0.3em] text-foreground tabular-nums outline-none transition placeholder:opacity-40 focus:border-primary focus:ring-2 focus:ring-primary/25"
              />
            </div>

            {error && (
              <div
                role="alert"
                className="mb-4 rounded-lg border border-danger/40 bg-danger/10 px-4 py-3 text-sm text-danger"
              >
                {error}
              </div>
            )}

            <button
              type="submit"
              disabled={code.length < 9 || status === "loading"}
              className="inline-flex h-11 w-full items-center justify-center rounded-lg bg-primary px-4 text-sm font-semibold text-primary-foreground transition hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
            >
              {status === "loading" ? "Authorizing..." : "Authorize Device"}
            </button>

            <p className="mt-4 text-center text-sm text-muted-foreground">
              Signed in as <span className="font-mono font-medium text-foreground tabular-nums">{user.username}</span>
            </p>
          </form>
        )}
      </Panel>
    </main>
  );
}
