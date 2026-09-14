import type { Locale } from "./locale";

const en: Record<string, string> = {
  "nav.leaderboard": "Leaderboard",
  "nav.teamboard": "Teamboard",
  "nav.docs": "Docs",
  "nav.profile": "Profile",
  "nav.signIn": "Sign in",
  "nav.openMenu": "Open menu",
  "nav.language": "Language",
  "nav.english": "English",
  "nav.chinese": "中文",
  "nav.skipToContent": "Skip to content",
};

const zh: Record<string, string> = {
  "nav.leaderboard": "排行榜",
  "nav.teamboard": "团队榜",
  "nav.docs": "文档",
  "nav.profile": "资料",
  "nav.signIn": "登录",
  "nav.openMenu": "打开菜单",
  "nav.language": "语言",
  "nav.english": "English",
  "nav.chinese": "中文",
  "nav.skipToContent": "跳到正文",
};

const dictionaries: Record<Locale, Record<string, string>> = { en, zh };

export function t(
  locale: Locale,
  key: string,
  vars?: Record<string, string | number>
): string {
  const template = dictionaries[locale][key] ?? dictionaries.en[key] ?? key;
  if (!vars) return template;
  return template.replace(/\{(\w+)\}/g, (_, name: string) =>
    Object.prototype.hasOwnProperty.call(vars, name)
      ? String(vars[name])
      : `{${name}}`
  );
}
