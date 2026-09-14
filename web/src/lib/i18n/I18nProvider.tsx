"use client";

import { createContext, useContext, useMemo } from "react";
import type { Locale } from "./locale";
import { t as translate } from "./t";
import { formatCompact, formatCurrency, formatNumber } from "@/lib/format";

const I18nContext = createContext<{
  locale: Locale;
  t: (key: string, vars?: Record<string, string | number>) => string;
}>({
  locale: "en",
  t: (key, vars) => translate("en", key, vars),
});

export function I18nProvider({
  locale,
  children,
}: {
  locale: Locale;
  children: React.ReactNode;
}) {
  const value = useMemo(
    () => ({
      locale,
      t: (key: string, vars?: Record<string, string | number>) =>
        translate(locale, key, vars),
    }),
    [locale]
  );
  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  return useContext(I18nContext);
}

export function useFormat() {
  const { locale } = useI18n();
  return useMemo(
    () => ({
      formatNumber: (value: number, compact = false) =>
        formatNumber(value, compact, locale),
      formatCurrency: (value: number, compact = false) =>
        formatCurrency(value, compact, locale),
      formatCompact: (value: number, kind: "number" | "currency") =>
        formatCompact(value, kind, locale),
    }),
    [locale]
  );
}
