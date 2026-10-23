"use client";

import React from "react";
import { ThemeProvider } from "next-themes";
import { I18nProvider, type Locale } from "@/lib/i18n";
import * as Tooltip from "@radix-ui/react-tooltip";

export function Providers({
  locale,
  children,
}: {
  locale: Locale;
  children: React.ReactNode;
}) {
  return (
    <I18nProvider locale={locale}>
      <ThemeProvider
        attribute="class"
        defaultTheme="system"
        enableSystem
        disableTransitionOnChange
      >
        <Tooltip.Provider delayDuration={150}>{children}</Tooltip.Provider>
      </ThemeProvider>
    </I18nProvider>
  );
}
