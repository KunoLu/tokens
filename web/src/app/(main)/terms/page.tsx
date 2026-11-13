import type { Metadata } from "next";
import {
  Bullets,
  Clause,
  CONTACT_EMAIL,
  LegalPage,
} from "@/components/legal/LegalPage";
import { getTranslator } from "@/lib/i18n/server";
import { SITE_HOST, SITE_URL } from "@/lib/site";

export async function generateMetadata(): Promise<Metadata> {
  const { t } = await getTranslator();

  return {
    title: `${t("legal.terms.title")} - Tokens`,
    description: t("legal.terms.description"),
    robots: { index: true, follow: true },
  };
};

const UPDATED = "2026-07-25";

export default async function TermsPage() {
  const { t } = await getTranslator();
  return (
    <LegalPage
      title={t("legal.terms.title")}
      description={t("legal.terms.description")}
      updated={UPDATED}
      englishHref="/terms"
    >
      <Clause heading={t("legal.terms.h.agreement")}>
        <p>
          {t("legal.terms.agreement.p1a")}
          <a href={SITE_URL}>{SITE_HOST}</a>
          {t("legal.terms.agreement.p1b")}
        </p>
      </Clause>

      <Clause heading={t("legal.terms.h.who")}>
        <p>{t("legal.terms.who.p1")}</p>
        <p>{t("legal.terms.who.p2")}</p>
      </Clause>

      <Clause heading={t("legal.terms.h.what")}>
        <p>{t("legal.terms.what.p1")}</p>
        <p>{t("legal.terms.what.p2")}</p>
      </Clause>

      <Clause heading={t("legal.terms.h.public")}>
        <p>{t("legal.terms.public.p1")}</p>
        <p>{t("legal.terms.public.p2")}</p>
        <p>{t("legal.terms.public.p3")}</p>
      </Clause>

      <Clause heading={t("legal.terms.h.honest")}>
        <p>{t("legal.terms.honest.p1")}</p>
        <Bullets
          items={[
            t("legal.terms.honest.1"),
            t("legal.terms.honest.2"),
            t("legal.terms.honest.3"),
            t("legal.terms.honest.4"),
            t("legal.terms.honest.5"),
          ]}
        />
        <p>{t("legal.terms.honest.p2")}</p>
      </Clause>

      <Clause heading={t("legal.terms.h.other")}>
        <Bullets
          items={[
            t("legal.terms.other.1"),
            t("legal.terms.other.2"),
            t("legal.terms.other.3"),
            t("legal.terms.other.4"),
          ]}
        />
        <p>
          {CONTACT_EMAIL ? (
            <>
              {t("legal.terms.other.p1a")}{" "}
              <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>{" "}
              {t("legal.terms.other.p1b")}
            </>
          ) : (
            t("legal.terms.other.p1NoEmail", { host: SITE_HOST })
          )}
        </p>
      </Clause>

      <Clause heading={t("legal.terms.h.enforce")}>
        <p>{t("legal.terms.enforce.p1")}</p>
        <p>{t("legal.terms.enforce.p2")}</p>
        <p>
          {CONTACT_EMAIL ? (
            <>
              {t("legal.terms.enforce.p3a")}{" "}
              <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>{" "}
              {t("legal.terms.enforce.p3b")}
            </>
          ) : (
            t("legal.terms.enforce.p3NoEmail", { host: SITE_HOST })
          )}
        </p>
      </Clause>

      <Clause heading={t("legal.terms.h.accuracy")}>
        <p>{t("legal.terms.accuracy.p1")}</p>
      </Clause>

      <Clause heading={t("legal.terms.h.software")}>
        <p>{t("legal.terms.software.p1", { host: SITE_HOST })}</p>
      </Clause>

      <Clause heading={t("legal.terms.h.avail")}>
        <p>{t("legal.terms.avail.p1")}</p>
        <p>{t("legal.terms.avail.p2")}</p>
      </Clause>

      <Clause heading={t("legal.terms.h.warranty")}>
        <p className="uppercase">{t("legal.terms.warranty.p1")}</p>
      </Clause>

      <Clause heading={t("legal.terms.h.liability")}>
        <p className="uppercase">{t("legal.terms.liability.p1")}</p>
        <p>{t("legal.terms.liability.p2")}</p>
      </Clause>

      <Clause heading={t("legal.terms.h.indemnity")}>
        <p>{t("legal.terms.indemnity.p1")}</p>
      </Clause>

      <Clause heading={t("legal.terms.h.law")}>
        <p>{t("legal.terms.law.p1")}</p>
      </Clause>

      <Clause heading={t("legal.terms.h.else")}>
        <p>
          {t("legal.terms.else.p1a")}{" "}
          <a href="/privacy">{t("legal.privacy.title")}</a>
          {t("legal.terms.else.p1b")}
        </p>
      </Clause>

      <Clause heading={t("legal.terms.h.contact")}>
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
