"use client";

import type { TokenContributionData, GraphColorPalette } from "@/lib/types";
import { getDarkGradeColors } from "@/lib/themes";
import {
  cn,
  formatCurrency,
  formatTokenCount,
  calculateCurrentStreak,
  calculateLongestStreak,
  findBestDay,
} from "@/lib/utils";
import { formatContributionDate } from "@/lib/date-utils";
import { formatDuration } from "@/lib/format";
import { useI18n, intlTag } from "@/lib/i18n";

interface StatsPanelProps {
  data: TokenContributionData;
  palette: GraphColorPalette;
  totalActiveTimeMs?: number | null;
  sessionCount?: number | null;
  mcpServers?: string[];
}

function BadgeList({ label, items, palette }: { label: string; items: string[]; palette: GraphColorPalette }) {
  return (
    <div className="mt-6 flex flex-wrap items-center gap-2 border-t border-border pt-6">
      <span className="mr-3 text-xs font-semibold uppercase tracking-wider text-muted-foreground max-[480px]:mr-0 max-[480px]:w-full">
        {label}:
      </span>
      {items.map((item) => (
        <span
          key={item}
          style={{ backgroundColor: `${palette.grade3}20` }}
          className="min-w-0 max-w-full truncate rounded-full px-3 py-1.5 text-xs font-medium text-foreground transition-all duration-200 hover:scale-105"
        >
          {item}
        </span>
      ))}
    </div>
  );
}

export function StatsPanel({ data, palette, totalActiveTimeMs, sessionCount, mcpServers }: StatsPanelProps) {
  const { t, locale } = useI18n();
  const { summary, contributions } = data;
  const currentStreak = calculateCurrentStreak(contributions);
  const longestStreak = calculateLongestStreak(contributions);
  const bestDay = findBestDay(contributions);

  return (
    <div className="rounded-2xl border border-border bg-card p-6 shadow-sm transition-shadow duration-150 hover:shadow-md">
      <h3 className="mb-4 text-sm font-bold uppercase tracking-wider text-muted-foreground">
        {t("graph.statistics")}
      </h3>

      {/* 2 up on phones, 1 below 400px, 3 from sm, 4 from md — the same
          breakpoints the styled grid used. */}
      <div className="grid grid-cols-2 gap-6 max-[560px]:gap-4 max-[400px]:grid-cols-1 sm:grid-cols-3 md:grid-cols-4">
        <StatItem
          label={t("graph.totalCost")}
          value={formatCurrency(summary.totalCost, locale)}
          highlightDarkColor={getDarkGradeColors(palette)[3]}
          highlightLightColor={palette.grade4}
          highlight
        />
        <StatItem label={t("graph.totalTokens")} value={formatTokenCount(summary.totalTokens, locale)} />
        <StatItem label={t("graph.activeDays")} value={`${summary.activeDays.toLocaleString(intlTag(locale))} / ${summary.totalDays.toLocaleString(intlTag(locale))}`} />
        <StatItem label={t("graph.avgPerDay")} value={formatCurrency(summary.averagePerDay, locale)} />
        <StatItem label={t("graph.currentStreak")} value={t(currentStreak === 1 ? "graph.daysOne" : "graph.daysMany", { n: currentStreak })} />
        <StatItem label={t("graph.longestStreak")} value={t(longestStreak === 1 ? "graph.daysOne" : "graph.daysMany", { n: longestStreak })} />
        {bestDay && bestDay.totals.cost > 0 && (
          <StatItem label={t("graph.bestDay")} value={formatContributionDate(bestDay, locale)} subValue={formatCurrency(bestDay.totals.cost, locale)} />
        )}
        <StatItem label={t("graph.models")} value={summary.models.length.toLocaleString(intlTag(locale))} />
        {totalActiveTimeMs != null && totalActiveTimeMs > 0 && (
          <StatItem label={t("graph.activeTime")} value={formatDuration(totalActiveTimeMs)} />
        )}
        {sessionCount != null && sessionCount > 0 && (
          <StatItem label={t("graph.sessions")} value={sessionCount.toLocaleString(intlTag(locale))} />
        )}
      </div>

      <BadgeList label={t("graph.clients")} items={summary.clients} palette={palette} />
      {mcpServers && mcpServers.length > 0 && (
        <BadgeList label={t("graph.mcps")} items={mcpServers} palette={palette} />
      )}
    </div>
  );
}

interface StatItemProps {
  label: string;
  value: string;
  subValue?: string;
  highlightDarkColor?: string;
  highlightLightColor?: string;
  highlight?: boolean;
}

function StatItem({
  label,
  value,
  subValue,
  highlightDarkColor,
  highlightLightColor,
  highlight,
}: StatItemProps) {
  return (
    <div className="flex min-w-0 flex-col gap-1">
      <div className="text-xs font-semibold uppercase tracking-wider text-muted-foreground [overflow-wrap:anywhere]">
        {label}
      </div>
      {/* The highlight colour differs per theme, and both values are runtime
          palette values, so they ride in as custom properties rather than as
          two impossible-to-generate Tailwind classes. Unhighlighted stats fall
          back to the foreground on both sides. */}
      <div
        style={
          {
            "--stat": highlight && highlightLightColor ? highlightLightColor : "var(--foreground)",
            "--stat-dark": highlight && highlightDarkColor ? highlightDarkColor : "var(--foreground)",
          } as React.CSSProperties
        }
        className={cn(
          "min-w-0 font-bold tracking-tight text-[var(--stat)] dark:text-[var(--stat-dark)] [overflow-wrap:anywhere]",
          highlight ? "text-xl max-[400px]:text-lg" : "text-lg max-[400px]:text-base"
        )}
      >
        {value}
      </div>
      {subValue && (
        <div className="text-xs font-medium text-muted-foreground">{subValue}</div>
      )}
    </div>
  );
}
