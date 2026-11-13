import { CONTAINER } from "@/components/layout/Container";
import { PageHeader } from "@/components/layout/PageHeader";
import { getTranslator } from "@/lib/i18n/server";
import { DEFAULT_LOCALE, LOCALE_TAGS, type Locale } from "@/lib/i18n/config";
import { richText } from "@/lib/i18n/rich";
import { cn } from "@/lib/utils";

/** 联系邮箱由部署方配置；未配置时法律页不渲染 mailto 链接（不造假地址）。 */
export const CONTACT_EMAIL = process.env.CONTACT_EMAIL || undefined;

function formatUpdated(updated: string, locale: Locale): string {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(updated)) return updated;
  // English legal pages traditionally read "25 July 2026"; every other locale
  // gets its own `Intl` ordering.
  const tag = locale === DEFAULT_LOCALE ? "en-GB" : LOCALE_TAGS[locale].lang;
  return new Intl.DateTimeFormat(tag, {
    day: "numeric",
    month: "long",
    year: "numeric",
    timeZone: "UTC",
  }).format(new Date(`${updated}T00:00:00Z`));
}

/**
 * Shared shell for the policy pages.
 *
 * Same width and heading rhythm as /docs so these do not read as bolted on,
 * but with a plain prose column: legal text is read top to bottom, not
 * scanned, so it gets no cards, no grid and no accent colour.
 */
export async function LegalPage({
  title,
  description,
  updated,
  englishHref,
  children,
}: {
  title: string;
  description: string;
  updated: string;
  /** The unprefixed (English, binding) address of this policy. */
  englishHref: string;
  children: React.ReactNode;
}) {
  const { locale, t } = await getTranslator();
  return (
    <main className={cn(CONTAINER, "pb-24 pt-10 sm:pt-14")} id="main-content">
      <div className="mx-auto w-full max-w-[720px]">
        <PageHeader title={title} description={description} />
        <p className="-mt-2 text-xs text-muted-foreground">
          {t("legal.lastUpdated", { date: formatUpdated(updated, locale) })}
        </p>
        {/* A translation is a courtesy, not the agreement: the English text
            is the one that holds, so every other language says so up front. */}
        {locale !== DEFAULT_LOCALE && (
          <p className="mt-4 rounded-md border bg-muted px-4 py-3 text-sm text-muted-foreground">
            {richText(t("legal.translationNotice"), {
              english: (
                <a
                  href={englishHref}
                  hrefLang="en"
                  className="underline underline-offset-2 hover:text-foreground"
                >
                  {t("legal.englishVersion")}
                </a>
              ),
            })}
          </p>
        )}
        <div className="mt-8 flex flex-col gap-8">{children}</div>
      </div>
    </main>
  );
}

export function Clause({
  heading,
  children,
}: {
  heading: string;
  children: React.ReactNode;
}) {
  return (
    <section className="flex flex-col gap-2.5">
      <h2 className="text-base font-semibold tracking-tight">{heading}</h2>
      <div className="flex flex-col gap-3 text-sm leading-relaxed text-muted-foreground [&_a]:underline [&_a]:underline-offset-2 [&_a:hover]:text-foreground">
        {children}
      </div>
    </section>
  );
}

export function Bullets({ items }: { items: React.ReactNode[] }) {
  return (
    <ul className="flex list-disc flex-col gap-1.5 pl-5">
      {items.map((item, i) => (
        <li key={i}>{item}</li>
      ))}
    </ul>
  );
}
