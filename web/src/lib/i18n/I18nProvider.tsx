"use client";

import { createContext, useContext, useMemo } from "react";
import type { Locale } from "./locale";
import { t as translate } from "./t";
import type { TranslationKey } from "./t";
import { formatCompact, formatCurrency, formatNumber } from "@/lib/format";

export type TranslationVariables = Record<string, string | number>;

export type Translate = (
  key: TranslationKey,
  vars?: TranslationVariables,
) => string;

const I18nContext = createContext<{
  locale: Locale;
  t: Translate;
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
      t: (key: TranslationKey, vars?: TranslationVariables) =>
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
