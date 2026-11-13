import type { Metadata } from "next";
import {
  Bullets,
  Clause,
  CONTACT_EMAIL,
  LegalPage,
} from "@/components/legal/LegalPage";
import { getTranslator } from "@/lib/i18n/server";
import { SITE_HOST, SITE_URL } from "@/lib/site";

export const metadata: Metadata = {
  title: "Privacy Policy - Tokens",
  description:
    "What Tokens collects, why, who it goes to, and how to get it deleted.",
  robots: { index: true, follow: true },
};

const UPDATED = "2026-07-25";

export default async function PrivacyPage() {
  const { t } = await getTranslator();
  return (
    <LegalPage
      title={t("legal.privacy.title")}
      description={t("legal.privacy.description")}
      updated={UPDATED}
      englishHref="/privacy"
    >
      <Clause heading={t("legal.privacy.h.short")}>
        <p>{t("legal.privacy.short.p1")}</p>
        <p>{t("legal.privacy.short.p2")}</p>
        <p>
          {t("legal.privacy.short.p3a")}{" "}
          <code className="font-mono text-[13px]">tokens submit --dry-run</code>{" "}
          {t("legal.privacy.short.p3b")}
        </p>
      </Clause>

      <Clause heading={t("legal.privacy.h.who")}>
        <p>
          Tokens (<a href={SITE_URL}>{SITE_HOST}</a>)
          {CONTACT_EMAIL ? (
            <>
              {" "}
              {t("legal.privacy.who.p1")}{" "}
              <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>.
            </>
          ) : (
            "."
          )}
        </p>
      </Clause>

      <Clause heading={t("legal.privacy.h.collect")}>
        <p>
          <strong className="font-medium text-foreground">
            {t("legal.privacy.collect.fromYouLabel")}
          </strong>{" "}
          {t("legal.privacy.collect.fromYou")}
        </p>
        <p>
          <strong className="font-medium text-foreground">
            {t("legal.privacy.collect.fromCliLabel")}
          </strong>{" "}
          {t("legal.privacy.collect.fromCli")}
        </p>
        <p>
          <strong className="font-medium text-foreground">
            {t("legal.privacy.collect.sessionLabel")}
          </strong>{" "}
          {t("legal.privacy.collect.session")}
        </p>
        <p>
          <strong className="font-medium text-foreground">
            {t("legal.privacy.collect.bannedLabel")}
          </strong>{" "}
          {t("legal.privacy.collect.banned")}
        </p>
      </Clause>

      <Clause heading={t("legal.privacy.h.never")}>
        <Bullets
          items={[
            t("legal.privacy.never.1"),
            t("legal.privacy.never.2"),
            t("legal.privacy.never.3"),
            t("legal.privacy.never.4"),
          ]}
        />
      </Clause>

      <Clause heading={t("legal.privacy.h.why")}>
        <Bullets
          items={[
            t("legal.privacy.why.1"),
            t("legal.privacy.why.2"),
            t("legal.privacy.why.3"),
            t("legal.privacy.why.4"),
          ]}
        />
        <p>{t("legal.privacy.why.p")}</p>
      </Clause>

      <Clause heading={t("legal.privacy.h.public")}>
        <p>{t("legal.privacy.public.p1")}</p>
        <p>{t("legal.privacy.public.p2")}</p>
        <p>{t("legal.privacy.public.p3")}</p>
      </Clause>

      <Clause heading={t("legal.privacy.h.cookies")}>
        <p>
          {t("legal.privacy.cookies.p1a")}{" "}
          <code className="font-mono text-[13px]">tt_session</code>
          {t("legal.privacy.cookies.p1b")}
        </p>
        <p>{t("legal.privacy.cookies.p2")}</p>
      </Clause>

      <Clause heading={t("legal.privacy.h.share")}>
        <p>{t("legal.privacy.share.p1")}</p>
        <p>{t("legal.privacy.share.p2")}</p>
        <Bullets
          items={[
            <>
              <strong className="font-medium text-foreground">
                Cloudflare
              </strong>{" "}
              {t("legal.privacy.share.cloudflare")}
            </>,
            <>
              <strong className="font-medium text-foreground">Neon</strong>{" "}
              {t("legal.privacy.share.neon")}
            </>,
            <>
              <strong className="font-medium text-foreground">Resend</strong>{" "}
              {t("legal.privacy.share.resend")}
            </>,
            <>
              <strong className="font-medium text-foreground">GitHub</strong>{" "}
              {t("legal.privacy.share.github")}
            </>,
          ]}
        />
        <p>{t("legal.privacy.share.p3")}</p>
      </Clause>

      <Clause heading={t("legal.privacy.h.where")}>
        <p>{t("legal.privacy.where.p1")}</p>
      </Clause>

      <Clause heading={t("legal.privacy.h.keep")}>
        <p>{t("legal.privacy.keep.p1")}</p>
        <p>{t("legal.privacy.keep.p2")}</p>
      </Clause>

      <Clause heading={t("legal.privacy.h.choices")}>
        <p>{t("legal.privacy.choices.intro")}</p>
        <Bullets
          items={[
            <>
              <strong className="font-medium text-foreground">
                {t("legal.privacy.choices.deleteUsageLabel")}
              </strong>{" "}
              {t("legal.privacy.choices.deleteUsage")}{" "}
              <code className="font-mono text-[13px]">
                tokens delete-submitted-data
              </code>{" "}
              {t("legal.privacy.choices.deleteUsageCli")}
            </>,
            <>
              <strong className="font-medium text-foreground">
                {t("legal.privacy.choices.deleteAccountLabel")}
              </strong>{" "}
              {t("legal.privacy.choices.deleteAccount")}
            </>,
            <>
              <strong className="font-medium text-foreground">
                {t("legal.privacy.choices.revokeLabel")}
              </strong>{" "}
              {t("legal.privacy.choices.revoke")}
            </>,
            <>
              <strong className="font-medium text-foreground">
                {t("legal.privacy.choices.stopLabel")}
              </strong>{" "}
              {t("legal.privacy.choices.stop")}
            </>,
          ]}
        />
      </Clause>

      <Clause heading={t("legal.privacy.h.ca")}>
        <p>{t("legal.privacy.ca.p1")}</p>
        <p>
          {t("legal.privacy.ca.p2a")}{" "}
          <strong className="font-medium text-foreground">
            {t("legal.privacy.ca.p2b")}
          </strong>
          {t("legal.privacy.ca.p2c")}
        </p>
        <p>{t("legal.privacy.ca.p3")}</p>
        <p>
          {CONTACT_EMAIL ? (
            <>
              {t("legal.privacy.ca.p4a")}{" "}
              <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>
              {t("legal.privacy.ca.p4b")}
            </>
          ) : (
            t("legal.privacy.ca.p4NoEmail", { host: SITE_HOST })
          )}
        </p>
      </Clause>

      <Clause heading={t("legal.privacy.h.states")}>
        <p>
          {CONTACT_EMAIL
            ? t("legal.privacy.states.p1")
            : t("legal.privacy.states.p1NoEmail", { host: SITE_HOST })}
        </p>
      </Clause>

      <Clause heading={t("legal.privacy.h.children")}>
        <p>
          {CONTACT_EMAIL ? (
            <>
              {t("legal.privacy.children.p1a")}{" "}
              <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>{" "}
              {t("legal.privacy.children.p1b")}
            </>
          ) : (
            t("legal.privacy.children.p1NoEmail", { host: SITE_HOST })
          )}
        </p>
      </Clause>

      <Clause heading={t("legal.privacy.h.security")}>
        <p>{t("legal.privacy.security.p1")}</p>
      </Clause>

      <Clause heading={t("legal.privacy.h.changes")}>
        <p>{t("legal.privacy.changes.p1")}</p>
      </Clause>

      <Clause heading={t("legal.privacy.h.contact")}>
        <p>
          {CONTACT_EMAIL ? (
            <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>
          ) : (
            <a href={SITE_URL}>{SITE_HOST}</a>
          )}
        </p>
      </Clause>
    </LegalPage>
  );
}
