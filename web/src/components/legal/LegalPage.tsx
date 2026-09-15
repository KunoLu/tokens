import { cookies } from "next/headers";
import { CONTAINER } from "@/components/layout/Container";
import { PageHeader } from "@/components/layout/PageHeader";
import { intlTag, LOCALE_COOKIE, parseLocale, t, type Locale } from "@/lib/i18n";
import { cn } from "@/lib/utils";

export const CONTACT_EMAIL = "hi@tokens.ci";

function formatUpdated(updated: string, locale: Locale): string {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(updated)) return updated;
  const tag = locale === "zh" ? intlTag(locale) : "en-GB";
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
  children,
}: {
  title: string;
  description: string;
  updated: string;
  children: React.ReactNode;
}) {
  const locale = parseLocale((await cookies()).get(LOCALE_COOKIE)?.value);
  return (
    <main className={cn(CONTAINER, "pb-24 pt-10 sm:pt-14")} id="main-content">
      <div className="mx-auto w-full max-w-[720px]">
        <PageHeader title={title} description={description} />
        <p className="-mt-2 text-xs text-muted-foreground">
          {t(locale, "legal.updated", { date: formatUpdated(updated, locale) })}
        </p>
        {locale === "zh" && (
          <p className="mt-4 rounded-md border bg-muted px-4 py-3 text-sm text-muted-foreground">
            {t(locale, "legal.enPrevails")}
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
