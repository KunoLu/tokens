import { cookies } from "next/headers";
import { intlTag, LOCALE_COOKIE, parseLocale, t } from "@/lib/i18n";

export interface BannedProfileData {
  banned: true;
  bannedAt: string | null;
  banReason: string | null;
  user: {
    username: string;
    displayName: string | null;
    avatarUrl: string | null;
    createdAt: string;
  };
}

/**
 * Replacement for the normal profile when the account is banned. The URL
 * stays reachable, but every statistic is withheld: the page renders only
 * the identity, a ban stamp, and the reason. The identity block is pushed
 * to grayscale so the stamp is the only thing with any color.
 */
export default async function BannedProfileView({ data }: { data: BannedProfileData }) {
  const locale = parseLocale((await cookies()).get(LOCALE_COOKIE)?.value);
  const { user } = data;
  const banDate = data.bannedAt
    ? new Intl.DateTimeFormat(intlTag(locale), {
        day: "numeric",
        month: "long",
        year: "numeric",
        timeZone: "UTC",
      }).format(new Date(data.bannedAt))
    : null;

  return (
    <main className="main-container" id="main-content">
      <div className="mx-auto max-w-[560px] px-4 py-16 sm:px-6">
        <section className="relative overflow-hidden rounded-2xl border border-border bg-card p-8 text-center">
          {/* Rotated rubber-stamp mark — deliberately the only colored element */}
          <div
            aria-hidden="true"
            className="pointer-events-none absolute right-4 top-6 rotate-12 rounded-md border-4 border-danger/70 px-3 py-1 font-mono text-xl font-black uppercase tracking-widest text-danger/80"
          >
            {t(locale, "banned.stamp")}
          </div>

          <div className="grayscale">
            {user.avatarUrl ? (
              // eslint-disable-next-line @next/next/no-img-element
              <img
                src={user.avatarUrl}
                alt={user.username}
                width={96}
                height={96}
                className="mx-auto h-24 w-24 rounded-2xl object-cover opacity-60 ring-1 ring-border"
              />
            ) : (
              <div className="mx-auto grid h-24 w-24 place-items-center rounded-2xl bg-foreground/10 font-mono text-3xl font-bold text-muted-foreground ring-1 ring-border">
                {user.username[0]?.toUpperCase() ?? "?"}
              </div>
            )}

            <h1 className="mt-5 font-mono text-xl font-bold text-muted-foreground line-through">
              @{user.username}
            </h1>
          </div>

          <p className="mt-2 inline-block rounded-md bg-danger/10 px-2.5 py-1 font-mono text-xs font-semibold uppercase tracking-wide text-danger">
            {banDate ? t(locale, "banned.badgeOn", { date: banDate }) : t(locale, "banned.badge")}
          </p>

          <p className="mt-6 text-sm leading-relaxed text-muted-foreground">
            {t(locale, "banned.desc")}
          </p>

          {data.banReason && (
            <div className="mt-6 rounded-xl border border-danger/25 bg-danger/5 p-4 text-left">
              <h2 className="font-mono text-[11px] font-semibold uppercase tracking-wide text-danger">
                {t(locale, "banned.reason")}
              </h2>
              <p className="mt-2 text-sm leading-relaxed text-foreground/85">
                {data.banReason}
              </p>
            </div>
          )}

        </section>
      </div>
    </main>
  );
}
