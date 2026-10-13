"use client";

import { useCallback, useEffect, useId, useState } from "react";
import { useI18n } from "@/lib/i18n";

const Section = "mb-6 rounded-xl border bg-card p-6";
const SectionTitle = "mb-4 text-lg font-semibold text-foreground";

interface ReadTokenRow {
  id: string;
  name: string;
  createdAt: string;
  lastUsedAt: string | null;
}

interface IssuedReadToken extends ReadTokenRow {
  token: string;
}

export default function PrivacySection() {
  const { t } = useI18n();
  const errorId = useId();
  const [isPrivate, setIsPrivate] = useState<boolean | null>(null);
  const [privacyLoadFailed, setPrivacyLoadFailed] = useState(false);
  const [isSavingPrivacy, setIsSavingPrivacy] = useState(false);
  const [tokens, setTokens] = useState<ReadTokenRow[]>([]);
  const [tokensLoadFailed, setTokensLoadFailed] = useState(false);
  const [tokenName, setTokenName] = useState("");
  const [createdToken, setCreatedToken] = useState<IssuedReadToken | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadPrivacy = useCallback(async () => {
    setPrivacyLoadFailed(false);
    const response = await fetch("/api/settings/privacy");
    if (!response.ok) {
      setPrivacyLoadFailed(true);
      return;
    }
    const data = (await response.json()) as { private?: unknown };
    setIsPrivate(data.private === true);
  }, []);

  const loadTokens = useCallback(async () => {
    setTokensLoadFailed(false);
    const response = await fetch("/api/settings/read-tokens");
    if (!response.ok) {
      setTokensLoadFailed(true);
      return;
    }
    const data = (await response.json()) as { tokens?: ReadTokenRow[] };
    setTokens(Array.isArray(data.tokens) ? data.tokens : []);
  }, []);

  useEffect(() => {
    void Promise.all([loadPrivacy(), loadTokens()]).catch(() => {
      setPrivacyLoadFailed(true);
      setTokensLoadFailed(true);
    });
  }, [loadPrivacy, loadTokens]);

  const togglePrivacy = async () => {
    if (isPrivate === null || isSavingPrivacy) return;
    setIsSavingPrivacy(true);
    setError(null);
    try {
      const response = await fetch("/api/settings/privacy", {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ private: !isPrivate }),
      });
      const data = (await response.json()) as { private?: unknown };
      if (!response.ok || typeof data.private !== "boolean") {
        setError(t("settings.privacy.saveFailed"));
        return;
      }
      setIsPrivate(data.private);
    } catch {
      setError(t("settings.privacy.saveFailed"));
    } finally {
      setIsSavingPrivacy(false);
    }
  };

  const createToken = async () => {
    setError(null);
    const response = await fetch("/api/settings/read-tokens", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name: tokenName }),
    });
    const data = (await response.json()) as { token?: IssuedReadToken };
    if (!response.ok || !data.token?.token) {
      setError(t("settings.privacy.tokenFailed"));
      return;
    }
    setCreatedToken(data.token);
    setTokenName("");
    await loadTokens();
  };

  const revokeToken = async (tokenId: string) => {
    setError(null);
    const response = await fetch(`/api/settings/read-tokens/${tokenId}`, { method: "DELETE" });
    if (!response.ok) {
      setError(t("settings.privacy.tokenFailed"));
      return;
    }
    setTokens((current) => current.filter((token) => token.id !== tokenId));
  };

  return (
    <section className={Section}>
      <h2 className={SectionTitle}>{t("settings.privacy.title")}</h2>
      <p className="mb-4 text-sm text-muted-foreground">{t("settings.privacy.desc")}</p>
      {privacyLoadFailed ? (
        <button type="button" className="text-sm underline" onClick={() => void loadPrivacy()}>
          {t("settings.privacy.retry")}
        </button>
      ) : (
        <div className="mb-6 flex items-center justify-between gap-4">
          <div>
            <p className="text-sm font-medium">{t("settings.privacy.switchLabel")}</p>
            <p className="text-sm text-muted-foreground">
              {isPrivate ? t("settings.privacy.on") : t("settings.privacy.off")}
            </p>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={isPrivate === true}
            disabled={isPrivate === null || isSavingPrivacy}
            onClick={() => void togglePrivacy()}
            className="h-10 cursor-pointer rounded-md border px-3.5 text-sm font-semibold disabled:opacity-60"
          >
            {isPrivate ? t("settings.privacy.turnOff") : t("settings.privacy.turnOn")}
          </button>
        </div>
      )}

      <h3 className="mb-2 text-sm font-semibold">{t("settings.privacy.tokensTitle")}</h3>
      <p className="mb-4 text-sm text-muted-foreground">{t("settings.privacy.tokensDesc")}</p>
      {createdToken ? (
        <p className="mb-4 break-all rounded-md border border-[var(--success)] p-3 font-mono text-sm">
          {createdToken.token}
        </p>
      ) : null}
      <div className="mb-4 grid grid-cols-[1fr_auto] gap-3 max-[560px]:grid-cols-1">
        <input
          value={tokenName}
          onChange={(event) => setTokenName(event.target.value)}
          placeholder={t("settings.privacy.tokenName")}
          className="h-10 rounded-md border bg-background px-3 text-sm"
          aria-describedby={error ? errorId : undefined}
        />
        <button
          type="button"
          onClick={() => void createToken()}
          className="h-10 cursor-pointer rounded-md border border-primary bg-primary px-3.5 text-sm font-semibold text-primary-foreground"
        >
          {t("settings.privacy.createToken")}
        </button>
      </div>
      {error ? (
        <p id={errorId} className="mb-3 text-sm text-[var(--danger)]">
          {error}
        </p>
      ) : null}
      {tokensLoadFailed ? (
        <button type="button" className="text-sm underline" onClick={() => void loadTokens()}>
          {t("settings.privacy.retry")}
        </button>
      ) : tokens.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("settings.privacy.noTokens")}</p>
      ) : (
        <ul className="space-y-2">
          {tokens.map((token) => (
            <li key={token.id} className="flex items-center justify-between gap-3 text-sm">
              <span>{token.name}</span>
              <button type="button" className="underline" onClick={() => void revokeToken(token.id)}>
                {t("settings.privacy.revoke")}
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
