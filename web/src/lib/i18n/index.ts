export type { Locale } from "./locale";
export type { TranslationKey } from "./t";
export type { Translate, TranslationVariables } from "./I18nProvider";
export {
  LOCALE_COOKIE,
  LOCALE_MAX_AGE,
  htmlLang,
  intlTag,
  localeCookieValue,
  parseLocale,
} from "./locale";
export { t, PALETTE_LABEL_KEYS } from "./t";
export {
  localizeServerError,
  localizeServerErrorFromCookie,
  localizeServerErrorList,
} from "./serverError";

export { I18nProvider, useFormat, useI18n } from "./I18nProvider";
