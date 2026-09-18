"use client";

import type { DailyContribution } from "@/lib/types";
import { formatCurrency, formatDateFull, formatNumber } from "@/lib/utils";
import { tw } from "@/lib/tw";
import { cn } from "@/lib/utils";
import { intlTag, useI18n } from "@/lib/i18n";

// Bar order is fixed Monday-first; the labels come from the dictionary.
const WEEKDAY_ORDER = [1, 2, 3, 4, 5, 6, 0] as const;

function weekdayIndex(date: string): number {
  const [year, month, day] = date.split("-").map(Number);
  return new Date(Date.UTC(year, month - 1, day)).getUTCDay();
}

export interface ProfileHabitsProps {
  contributions: DailyContribution[];
}

export function ProfileHabits({ contributions }: ProfileHabitsProps) {
  const { t, locale } = useI18n();
  const weekdayNames = t("profile.habits.weekdays").split(",");
  const weekdayShorts = t("profile.habits.weekdaysShort").split(",");
  const weekdayTokens = [0, 0, 0, 0, 0, 0, 0];
  let totalTokens = 0;
  let biggestDay: DailyContribution | null = null;

  for (const contribution of contributions) {
    const tokens = contribution.totals.tokens;
    if (tokens <= 0) continue;
    weekdayTokens[weekdayIndex(contribution.date)] += tokens;
    totalTokens += tokens;
    if (!biggestDay || tokens > biggestDay.totals.tokens) {
      biggestDay = contribution;
    }
  }

  if (!biggestDay || totalTokens <= 0) return null;

  const topWeekdayIndex = weekdayTokens.indexOf(Math.max(...weekdayTokens));
  const topWeekdayTokens = weekdayTokens[topWeekdayIndex];
  const topWeekdayShare = (topWeekdayTokens / totalTokens) * 100;

  return (
    <Panel aria-labelledby="profile-habits-title">
      <Header>
        <Title id="profile-habits-title">{t("profile.habits.title")}</Title>
        <Range>{t("profile.habits.range")}</Range>
      </Header>

      <Highlights>
        <Highlight>
          <Label>{t("profile.habits.mostProductive")}</Label>
          <Value>{weekdayNames[topWeekdayIndex]}</Value>
          <Meta title={topWeekdayTokens.toLocaleString(intlTag(locale))}>
            {t("profile.habits.ofTotal", { tokens: formatNumber(topWeekdayTokens, locale), share: topWeekdayShare.toFixed(0) })}
          </Meta>
        </Highlight>
        <Highlight>
          <Label>{t("profile.habits.biggestDay")}</Label>
          <Value $accent title={biggestDay.totals.tokens.toLocaleString(intlTag(locale))}>
            {t("tokens.count", { n: formatNumber(biggestDay.totals.tokens, locale) })}
          </Value>
          <Meta>
            {formatDateFull(biggestDay.date, locale)} · {formatCurrency(biggestDay.totals.cost, locale)}
          </Meta>
        </Highlight>
      </Highlights>

      <Distribution>
        <Label>{t("profile.habits.byWeekday")}</Label>
        <Bars>
          {WEEKDAY_ORDER.map((index) => {
            const tokens = weekdayTokens[index];
            const isTop = index === topWeekdayIndex;
            const height = topWeekdayTokens > 0
              ? Math.max((tokens / topWeekdayTokens) * 100, tokens > 0 ? 6 : 2)
              : 2;
            return (
              <BarColumn key={index}>
                <BarTrack>
                  <Bar
                    $height={height}
                    $top={isTop}
                    title={t("profile.habits.weekdayTitle", { weekday: weekdayNames[index], tokens: formatNumber(tokens, locale) })}
                  />
                </BarTrack>
                <BarLabel $top={isTop}>{weekdayShorts[index]}</BarLabel>
              </BarColumn>
            );
          })}
        </Bars>
      </Distribution>
    </Panel>
  );
}

const Panel = tw("section", "overflow-hidden rounded-xl border bg-card");

const Header = tw(
  "div",
  "flex items-baseline justify-between gap-3 border-b px-4 py-3.5"
);

const Title = tw("h2", "m-0 text-base font-medium text-foreground");
const Range = tw("span", "text-xs text-muted-foreground");

const Highlights = tw("div", "grid grid-cols-2 border-b");

// The divider is on the second cell rather than between them, which is what
// `& + &` did — the first column must not carry a left edge.
const Highlight = tw(
  "div",
  "min-w-0 px-4 py-3.5 [&+&]:border-l [&+&]:border-border"
);

const Label = tw(
  "p",
  "m-0 text-[11px] font-semibold uppercase tracking-[0.06em] text-muted-foreground"
);

const Value = ({
  $accent,
  className,
  ...props
}: React.ComponentPropsWithoutRef<"p"> & { $accent?: boolean }) => (
  <p
    {...props}
    className={cn(
      "m-0 mt-[5px] overflow-hidden text-ellipsis whitespace-nowrap text-base font-semibold [font-variant-numeric:tabular-nums]",
      $accent ? "text-primary" : "text-foreground",
      className
    )}
  />
);

const Meta = tw(
  "p",
  "m-0 mt-[3px] overflow-hidden text-ellipsis whitespace-nowrap text-xs text-muted-foreground [font-variant-numeric:tabular-nums]"
);

const Distribution = tw("div", "px-4 pb-4 pt-3.5");
const Bars = tw("div", "mt-2.5 flex items-end gap-2");
const BarColumn = tw("div", "flex min-w-0 flex-1 flex-col items-center gap-1.5");
const BarTrack = tw("div", "flex h-16 w-full items-end");

// Height is a percentage computed per bar, so it stays inline.
const Bar = ({
  $height,
  $top,
  style,
  ...props
}: React.ComponentPropsWithoutRef<"div"> & {
  $height: number;
  $top: boolean;
}) => (
  <div
    {...props}
    style={{ height: `${$height}%`, ...style }}
    className={cn(
      "w-full rounded-b-sm rounded-t",
      $top ? "bg-primary" : "bg-muted"
    )}
  />
);

const BarLabel = ({
  $top,
  className,
  ...props
}: React.ComponentPropsWithoutRef<"span"> & { $top: boolean }) => (
  <span
    {...props}
    className={cn(
      "text-[10px] font-medium",
      $top ? "text-foreground" : "text-muted-foreground",
      className
    )}
  />
);
