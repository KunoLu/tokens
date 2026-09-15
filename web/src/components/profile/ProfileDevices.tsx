"use client";

import { formatRelativeTime } from "@/lib/formatRelativeTime";
import { formatCurrency, formatNumber } from "@/lib/utils";
import {
  ListBody,
  ListCaption,
  ListCard,
  ListCell,
  ListHead,
  ListHeaderCell,
  ListPrimaryCell,
  ListRow,
  ListTable,
  NumericValue,
} from "./listStyles";
import { tw } from "@/lib/tw";
import { cn } from "@/lib/utils";
import { intlTag, useI18n } from "@/lib/i18n";

/** Public device usage shape returned by the profile devices route. */
export interface ProfileDevice {
  id: string;
  deviceKey: string;
  displayName: string;
  customName: string | null;
  createdAt: string | null;
  lastSubmittedAt: string | null;
  totalTokens: number;
  totalCost: number;
  inputTokens: number;
  outputTokens: number;
  activeDays: number;
  firstDay: string | null;
  lastDay: string | null;
}

export interface ProfileDevicesProps {
  devices: ProfileDevice[];
  className?: string;
}

const DevicesSection = tw(
  "section",
  "overflow-hidden rounded-xl border bg-card text-foreground"
);

const SectionHeading = tw(
  "h2",
  "m-0 border-b px-4 py-3.5 text-[0.9375rem] font-semibold leading-tight tracking-tight text-foreground"
);

const DevicesList = ({
  className,
  ...props
}: React.ComponentPropsWithoutRef<"div">) => (
  <ListCard {...props} className={cn("border-y-0", className)} />
);

const DeviceName = tw(
  "span",
  "block overflow-hidden text-ellipsis whitespace-nowrap text-foreground max-[639px]:whitespace-normal max-[639px]:[overflow-wrap:anywhere]"
);

const LastSubmitted = tw(
  "span",
  "mt-1 block text-xs font-normal leading-tight text-muted-foreground"
);
/** Compact per-device usage for public profiles. */
export function ProfileDevices({ devices, className }: ProfileDevicesProps) {
  const { t, locale } = useI18n();

  if (devices.length === 0) return null;

  return (
    <DevicesSection
      className={className}
      aria-labelledby="profile-devices-heading"
    >
      <SectionHeading id="profile-devices-heading">
        {t("profile.devices.heading")}
      </SectionHeading>

      <DevicesList>
        <ListTable>
          <ListCaption>{t("profile.devices.caption")}</ListCaption>
          <ListHead>
            <tr>
              <ListHeaderCell $width="52%">{t("profile.devices.device")}</ListHeaderCell>
              <ListHeaderCell $width="19%" $align="right">
                {t("profile.tokens")}
              </ListHeaderCell>
              <ListHeaderCell $width="16%" $align="right">
                {t("profile.cost")}
              </ListHeaderCell>
              <ListHeaderCell $width="13%" $align="right">
                {t("profile.activeDays")}
              </ListHeaderCell>
            </tr>
          </ListHead>
          <ListBody>
            {devices.map((device) => (
              <ListRow key={device.id}>
                <ListPrimaryCell scope="row">
                  <DeviceName>{device.displayName}</DeviceName>
                  <LastSubmitted>
                    {t("profile.devices.lastSubmittedPrefix")}{" "}
                    <time
                      dateTime={device.lastSubmittedAt ?? undefined}
                      suppressHydrationWarning
                    >
                      {formatRelativeTime(device.lastSubmittedAt, undefined, locale)}
                    </time>
                  </LastSubmitted>
                </ListPrimaryCell>
                <ListCell data-label={t("profile.tokens")} $align="right">
                  <NumericValue
                    title={device.totalTokens.toLocaleString(intlTag(locale))}
                  >
                    {formatNumber(device.totalTokens, locale)}
                  </NumericValue>
                </ListCell>
                <ListCell data-label={t("profile.cost")} $align="right">
                  <NumericValue $accent>
                    {formatCurrency(device.totalCost, locale)}
                  </NumericValue>
                </ListCell>
                <ListCell data-label={t("profile.activeDays")} $align="right">
                  <NumericValue>
                    {device.activeDays.toLocaleString(intlTag(locale))}
                  </NumericValue>
                </ListCell>
              </ListRow>
            ))}
          </ListBody>
        </ListTable>
      </DevicesList>
    </DevicesSection>
  );
}
