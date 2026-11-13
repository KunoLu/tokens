import type { Metadata } from "next";
import { CONTAINER } from "@/components/layout/Container";
import { PageHeader } from "@/components/layout/PageHeader";
import { buttonVariants } from "@/components/ui/button";

import { Card, CardContent } from "@/components/ui/card";
import { getSession } from "@/lib/auth/session";
import { getTranslator } from "@/lib/i18n/server";
import { localizeHref } from "@/lib/i18n/config";
import { loadTeamsPageData } from "@/lib/teams/pageData";
import { cn } from "@/lib/utils";
import { TeamsClient } from "@/components/teams/TeamsClient";

export const dynamic = "force-dynamic";

export async function generateMetadata(): Promise<Metadata> {
  const { t } = await getTranslator();

  return {
    title: `${t("teams.title")} - Tokens`,
    description: t("teams.desc"),
  };
};

export default async function TeamsPage() {
  const { locale, t } = await getTranslator();
  const session = await getSession();

  if (!session) {
    return (
      <main className={cn(CONTAINER, "pb-24 pt-10 sm:pt-14")} id="main-content">
        <div className="mx-auto w-full max-w-[860px]">
          <PageHeader
            title={t("teams.title")}
            description={t("teams.desc")}
          />
          <Card>
            <CardContent className="flex flex-col items-start gap-4">
              <p className="text-sm text-muted-foreground">
                {t("teams.gateDesc")}
              </p>
              <a
                href={`/login?returnTo=${encodeURIComponent(localizeHref("/teams", locale))}`}
                className={buttonVariants()}
              >
                {t("nav.signIn")}
              </a>


            </CardContent>
          </Card>
        </div>
      </main>
    );
  }

  const initialData = await loadTeamsPageData(session.id);
  return (
    <main className={cn(CONTAINER, "pb-24 pt-10 sm:pt-14")} id="main-content">
      <div className="mx-auto w-full max-w-[860px]">
        <TeamsClient initialData={initialData} selfId={session.id} />
      </div>
    </main>
  );
}
