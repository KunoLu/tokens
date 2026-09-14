export type Locale = "en" | "zh";

export const LOCALE_COOKIE = "tt_locale";
export const LOCALE_MAX_AGE = 365 * 24 * 60 * 60;

export function parseLocale(value: string | undefined | null): Locale {
  return value === "zh" ? "zh" : "en";
}

export function htmlLang(locale: Locale): string {
  return locale === "zh" ? "zh-CN" : "en";
}

export function intlTag(locale: Locale): string {
  return locale === "zh" ? "zh-CN" : "en-US";
}

export function localeCookieValue(locale: Locale): string {
  return `${LOCALE_COOKIE}=${locale}; Path=/; Max-Age=${LOCALE_MAX_AGE}; SameSite=Lax`;
}
