"use client";

import { useState } from "react";
import type { TokenContributionData } from "@/lib/types";
import { DataInput } from "@/components/DataInput";
import { GraphContainer } from "@/components/GraphContainer";
import { Panel } from "@/components/ui/primitives";
import { PageHeader } from "@/components/layout/PageHeader";
import { intlTag, useFormat, useI18n } from "@/lib/i18n";
import { formatDate } from "@/lib/utils";

export default function LocalClient() {
  const [data, setData] = useState<TokenContributionData | null>(null);
  const { t, locale } = useI18n();
  const { formatCurrency } = useFormat();

  return (
    // No min-h-screen: layout.tsx's body is already `min-h-dvh flex flex-col`
    // with this content in a `flex-1` wrapper, so adding a viewport height here
    // made the page 100vh *plus* the nav and footer.
    <main id="main-content" className="bg-background px-4 py-8 sm:px-6">
      <div className="mx-auto max-w-[820px]">
        <PageHeader
          title={t("local.title")}
          description={t("local.desc")}
        />

        {!data ? (
          <DataInput onDataLoaded={setData} />
        ) : (
          <div className="flex flex-col gap-6">
            <Panel className="p-4 sm:p-6">
              <div className="flex flex-wrap items-center gap-x-3 gap-y-2 text-sm">
                <span className="text-muted-foreground">{t("local.dataLoaded")}</span>
                <span className="font-mono tabular-nums font-semibold text-foreground">
                  {formatDate(data.meta.dateRange.start, locale)} - {formatDate(data.meta.dateRange.end, locale)}
                </span>
                <span className="text-border">|</span>
                <span className="font-mono tabular-nums font-semibold text-primary">
                  {t("local.total", { amount: formatCurrency(data.summary.totalCost) })}
                </span>
                <span className="text-border">|</span>
                <span className="text-muted-foreground">
                  <span className="font-mono tabular-nums">{data.summary.activeDays.toLocaleString(intlTag(locale))}</span>{" "}
                  {t("local.activeDays")}
                </span>
                <button
                  onClick={() => setData(null)}
                  className="ml-auto inline-flex h-10 items-center justify-center gap-1.5 rounded-lg border border-border bg-card px-4 text-sm font-medium text-foreground transition hover:border-foreground/20 hover:bg-muted"
                >
                  {t("local.loadDifferent")}
                </button>
              </div>
            </Panel>
            <GraphContainer data={data} />
          </div>
        )}
      </div>
    </main>
  );
}
