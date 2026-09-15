import type { Metadata } from "next";
import { cookies } from "next/headers";
import {
  Bullets,
  Clause,
  CONTACT_EMAIL,
  LegalPage,
} from "@/components/legal/LegalPage";
import { LOCALE_COOKIE, parseLocale, t } from "@/lib/i18n";

export const metadata: Metadata = {
  title: "Privacy Policy - Tokens",
  description:
    "What Tokens collects, why, who it goes to, and how to get it deleted.",
  robots: { index: true, follow: true },
};

const UPDATED = "2026-07-25";

export default async function PrivacyPage() {
  const locale = parseLocale((await cookies()).get(LOCALE_COOKIE)?.value);
  return (
    <LegalPage
      title={t(locale, "privacy.title")}
      description={t(locale, "privacy.desc")}
      updated={UPDATED}
    >
      <Clause heading={t(locale, "privacy.h.short")}>
        <p>{t(locale, "privacy.short.p1")}</p>
        <p>{t(locale, "privacy.short.p2")}</p>
        <p>
          {t(locale, "privacy.short.p3a")}{" "}
          <code className="font-mono text-[13px]">tokens submit --dry-run</code>{" "}
          {t(locale, "privacy.short.p3b")}
        </p>
      </Clause>

      <Clause heading={t(locale, "privacy.h.who")}>
        <p>
          Tokens (<a href="https://tokens.ci">tokens.ci</a>){" "}
          {t(locale, "privacy.who.p1")}{" "}
          <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>.
        </p>
      </Clause>

      <Clause heading={t(locale, "privacy.h.collect")}>
        <p>
          <strong className="font-medium text-foreground">
            {t(locale, "privacy.collect.fromYouLabel")}
          </strong>{" "}
          {t(locale, "privacy.collect.fromYou")}
        </p>
        <p>
          <strong className="font-medium text-foreground">
            {t(locale, "privacy.collect.fromCliLabel")}
          </strong>{" "}
          {t(locale, "privacy.collect.fromCli")}
        </p>
        <p>
          <strong className="font-medium text-foreground">
            {t(locale, "privacy.collect.sessionLabel")}
          </strong>{" "}
          {t(locale, "privacy.collect.session")}
        </p>
        <p>
          <strong className="font-medium text-foreground">
            {t(locale, "privacy.collect.bannedLabel")}
          </strong>{" "}
          {t(locale, "privacy.collect.banned")}
        </p>
      </Clause>

      <Clause heading={t(locale, "privacy.h.never")}>
        <Bullets
          items={[
            t(locale, "privacy.never.1"),
            t(locale, "privacy.never.2"),
            t(locale, "privacy.never.3"),
            t(locale, "privacy.never.4"),
          ]}
        />
      </Clause>

      <Clause heading={t(locale, "privacy.h.why")}>
        <Bullets
          items={[
            t(locale, "privacy.why.1"),
            t(locale, "privacy.why.2"),
            t(locale, "privacy.why.3"),
            t(locale, "privacy.why.4"),
          ]}
        />
        <p>{t(locale, "privacy.why.p")}</p>
      </Clause>

      <Clause heading={t(locale, "privacy.h.public")}>
        <p>{t(locale, "privacy.public.p1")}</p>
        <p>{t(locale, "privacy.public.p2")}</p>
        <p>{t(locale, "privacy.public.p3")}</p>
      </Clause>

      <Clause heading={t(locale, "privacy.h.cookies")}>
        <p>
          {t(locale, "privacy.cookies.p1a")}{" "}
          <code className="font-mono text-[13px]">tt_session</code>
          {t(locale, "privacy.cookies.p1b")}
        </p>
        <p>{t(locale, "privacy.cookies.p2")}</p>
      </Clause>

      <Clause heading={t(locale, "privacy.h.share")}>
        <p>{t(locale, "privacy.share.p1")}</p>
        <p>{t(locale, "privacy.share.p2")}</p>
        <Bullets
          items={[
            <>
              <strong className="font-medium text-foreground">
                Cloudflare
              </strong>{" "}
              {t(locale, "privacy.share.cloudflare")}
            </>,
            <>
              <strong className="font-medium text-foreground">Neon</strong>{" "}
              {t(locale, "privacy.share.neon")}
            </>,
            <>
              <strong className="font-medium text-foreground">Resend</strong>{" "}
              {t(locale, "privacy.share.resend")}
            </>,
            <>
              <strong className="font-medium text-foreground">GitHub</strong>{" "}
              {t(locale, "privacy.share.github")}
            </>,
          ]}
        />
        <p>{t(locale, "privacy.share.p3")}</p>
      </Clause>

      <Clause heading={t(locale, "privacy.h.where")}>
        <p>{t(locale, "privacy.where.p1")}</p>
      </Clause>

      <Clause heading={t(locale, "privacy.h.keep")}>
        <p>{t(locale, "privacy.keep.p1")}</p>
        <p>{t(locale, "privacy.keep.p2")}</p>
      </Clause>

      <Clause heading={t(locale, "privacy.h.choices")}>
        <p>{t(locale, "privacy.choices.intro")}</p>
        <Bullets
          items={[
            <>
              <strong className="font-medium text-foreground">
                {t(locale, "privacy.choices.deleteUsageLabel")}
              </strong>{" "}
              {t(locale, "privacy.choices.deleteUsage")}{" "}
              <code className="font-mono text-[13px]">
                tokens delete-submitted-data
              </code>{" "}
              {t(locale, "privacy.choices.deleteUsageCli")}
            </>,
            <>
              <strong className="font-medium text-foreground">
                {t(locale, "privacy.choices.deleteAccountLabel")}
              </strong>{" "}
              {t(locale, "privacy.choices.deleteAccount")}
            </>,
            <>
              <strong className="font-medium text-foreground">
                {t(locale, "privacy.choices.revokeLabel")}
              </strong>{" "}
              {t(locale, "privacy.choices.revoke")}
            </>,
            <>
              <strong className="font-medium text-foreground">
                {t(locale, "privacy.choices.stopLabel")}
              </strong>{" "}
              {t(locale, "privacy.choices.stop")}
            </>,
          ]}
        />
      </Clause>

      <Clause heading={t(locale, "privacy.h.ca")}>
        <p>{t(locale, "privacy.ca.p1")}</p>
        <p>
          {t(locale, "privacy.ca.p2a")}{" "}
          <strong className="font-medium text-foreground">
            {t(locale, "privacy.ca.p2b")}
          </strong>
          {t(locale, "privacy.ca.p2c")}
        </p>
        <p>{t(locale, "privacy.ca.p3")}</p>
        <p>
          {t(locale, "privacy.ca.p4a")}{" "}
          <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>
          {t(locale, "privacy.ca.p4b")}
        </p>
      </Clause>

      <Clause heading={t(locale, "privacy.h.states")}>
        <p>{t(locale, "privacy.states.p1")}</p>
      </Clause>

      <Clause heading={t(locale, "privacy.h.children")}>
        <p>
          {t(locale, "privacy.children.p1a")}{" "}
          <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>{" "}
          {t(locale, "privacy.children.p1b")}
        </p>
      </Clause>

      <Clause heading={t(locale, "privacy.h.security")}>
        <p>{t(locale, "privacy.security.p1")}</p>
      </Clause>

      <Clause heading={t(locale, "privacy.h.changes")}>
        <p>{t(locale, "privacy.changes.p1")}</p>
      </Clause>

      <Clause heading={t(locale, "privacy.h.contact")}>
        <p>
          <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>
        </p>
      </Clause>
    </LegalPage>
  );
}
