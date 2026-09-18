import Link from "next/link";
import { cookies } from "next/headers";
import { LOCALE_COOKIE, parseLocale, t } from "@/lib/i18n";

export default async function ProfileNotFound() {
  const locale = parseLocale((await cookies()).get(LOCALE_COOKIE)?.value);
  return (
    <div className="flex min-h-screen flex-col bg-background pt-16">
      <main className="mx-auto w-full max-w-[800px] flex-1 px-6 py-10">
        <div className="py-20 text-center">
          <h1 className="mb-2 text-2xl font-bold text-foreground">{t(locale, "notFound.title")}</h1>
          <p className="mb-6 text-muted-foreground">{t(locale, "notFound.desc")}</p>
          <Link href="/leaderboard" className="inline-flex items-center rounded-lg bg-primary px-4 py-2 font-medium text-primary-foreground transition hover:opacity-90">
            {t(locale, "notFound.back")}
          </Link>
        </div>
      </main>
    </div>
  );
}
