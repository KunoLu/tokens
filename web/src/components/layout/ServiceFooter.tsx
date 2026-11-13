import Link from "next/link";
import { CONTAINER } from "@/components/layout/Container";
import { cn } from "@/lib/utils";
import { getTranslator } from "@/lib/i18n/server";
import { localizeHref } from "@/lib/i18n/config";

/**
 * Site footer: one centred row — site identity, the policy links, and the
 * upstream credit (this fork is MIT-licensed from Tokscale; the attribution
 * is honest and cheap to keep). Nothing about hosting or sponsors: those
 * describe upstream's deployment, not this one.
 */
export async function ServiceFooter() {
  // Read at render rather than hardcoded. This is a server component, so the
  // year comes from the server clock once and ships in the HTML — no hydration
  // mismatch, and no January where the site still claims the previous year.
  const year = new Date().getFullYear();
  const { locale, t } = await getTranslator();

  return (
    <footer className="mt-auto border-t" aria-label={t("footer.label")}>
      <div className={cn(CONTAINER, "flex justify-center py-7 text-center")}>
        {/* The policy links live here and nowhere else. They have to be
            reachable from every page, but they are not something anyone came
            for, so they sit at the same weight as the rest of the footer. */}
        <span className="text-xs text-muted-foreground">
          Tokens · © {year} ·{" "}
          <Link
            href={localizeHref("/privacy", locale)}
            className="transition-colors hover:text-foreground"
          >
            {t("footer.privacy")}
          </Link>{" "}
          ·{" "}
          <Link
            href={localizeHref("/terms", locale)}
            className="transition-colors hover:text-foreground"
          >
            {t("footer.terms")}
          </Link>{" "}
          ·{" "}
          <a
            href="https://github.com/junhoyeo/tokscale"
            target="_blank"
            rel="noopener noreferrer"
            className="transition-colors hover:text-foreground"
          >
            {t("footer.builtOn")}
          </a>
        </span>
      </div>
    </footer>
  );
}
