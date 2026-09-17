import type { Metadata } from "next";
import { cookies } from "next/headers";
import {
  Bullets,
  Clause,
  CONTACT_EMAIL,
  LegalPage,
} from "@/components/legal/LegalPage";
import { LOCALE_COOKIE, parseLocale, t } from "@/lib/i18n";
import { SITE_HOST, SITE_URL } from "@/lib/site";

export const metadata: Metadata = {
  title: "Terms of Service - Tokens",
  description: "The rules for using Tokens, and what we do and do not promise.",
  robots: { index: true, follow: true },
};

const UPDATED = "2026-07-25";

export default async function TermsPage() {
  const locale = parseLocale((await cookies()).get(LOCALE_COOKIE)?.value);
  return (
    <LegalPage
      title={t(locale, "terms.title")}
      description={t(locale, "terms.desc")}
      updated={UPDATED}
    >
      <Clause heading={t(locale, "terms.h.agreement")}>
        <p>
          {t(locale, "terms.agreement.p1a")}
          <a href={SITE_URL}>{SITE_HOST}</a>
          {t(locale, "terms.agreement.p1b")}
        </p>
      </Clause>

      <Clause heading={t(locale, "terms.h.who")}>
        <p>{t(locale, "terms.who.p1")}</p>
        <p>{t(locale, "terms.who.p2")}</p>
      </Clause>

      <Clause heading={t(locale, "terms.h.what")}>
        <p>{t(locale, "terms.what.p1")}</p>
        <p>{t(locale, "terms.what.p2")}</p>
      </Clause>

      <Clause heading={t(locale, "terms.h.public")}>
        <p>{t(locale, "terms.public.p1")}</p>
        <p>{t(locale, "terms.public.p2")}</p>
        <p>{t(locale, "terms.public.p3")}</p>
      </Clause>

      <Clause heading={t(locale, "terms.h.honest")}>
        <p>{t(locale, "terms.honest.p1")}</p>
        <Bullets
          items={[
            t(locale, "terms.honest.1"),
            t(locale, "terms.honest.2"),
            t(locale, "terms.honest.3"),
            t(locale, "terms.honest.4"),
            t(locale, "terms.honest.5"),
          ]}
        />
        <p>{t(locale, "terms.honest.p2")}</p>
      </Clause>

      <Clause heading={t(locale, "terms.h.other")}>
        <Bullets
          items={[
            t(locale, "terms.other.1"),
            t(locale, "terms.other.2"),
            t(locale, "terms.other.3"),
            t(locale, "terms.other.4"),
          ]}
        />
        <p>
          {CONTACT_EMAIL ? (
            <>
              {t(locale, "terms.other.p1a")}{" "}
              <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>{" "}
              {t(locale, "terms.other.p1b")}
            </>
          ) : (
            t(locale, "terms.other.p1NoEmail", { host: SITE_HOST })
          )}
        </p>
      </Clause>

      <Clause heading={t(locale, "terms.h.enforce")}>
        <p>{t(locale, "terms.enforce.p1")}</p>
        <p>{t(locale, "terms.enforce.p2")}</p>
        <p>
          {CONTACT_EMAIL ? (
            <>
              {t(locale, "terms.enforce.p3a")}{" "}
              <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>{" "}
              {t(locale, "terms.enforce.p3b")}
            </>
          ) : (
            t(locale, "terms.enforce.p3NoEmail", { host: SITE_HOST })
          )}
        </p>
      </Clause>

      <Clause heading={t(locale, "terms.h.accuracy")}>
        <p>{t(locale, "terms.accuracy.p1")}</p>
      </Clause>

      <Clause heading={t(locale, "terms.h.software")}>
        <p>{t(locale, "terms.software.p1", { host: SITE_HOST })}</p>
      </Clause>

      <Clause heading={t(locale, "terms.h.avail")}>
        <p>{t(locale, "terms.avail.p1")}</p>
        <p>{t(locale, "terms.avail.p2")}</p>
      </Clause>

      <Clause heading={t(locale, "terms.h.warranty")}>
        <p className="uppercase">{t(locale, "terms.warranty.p1")}</p>
      </Clause>

      <Clause heading={t(locale, "terms.h.liability")}>
        <p className="uppercase">{t(locale, "terms.liability.p1")}</p>
        <p>{t(locale, "terms.liability.p2")}</p>
      </Clause>

      <Clause heading={t(locale, "terms.h.indemnity")}>
        <p>{t(locale, "terms.indemnity.p1")}</p>
      </Clause>

      <Clause heading={t(locale, "terms.h.law")}>
        <p>{t(locale, "terms.law.p1")}</p>
      </Clause>

      <Clause heading={t(locale, "terms.h.else")}>
        <p>
          {t(locale, "terms.else.p1a")}{" "}
          <a href="/privacy">{t(locale, "privacy.title")}</a>
          {t(locale, "terms.else.p1b")}
        </p>
      </Clause>

      <Clause heading={t(locale, "terms.h.contact")}>
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
