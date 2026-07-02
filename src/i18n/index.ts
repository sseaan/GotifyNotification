import { create } from "zustand";
import zh from "./zh-CN";
import en from "./en-US";

type DeepString<T> = {
  [K in keyof T]: T[K] extends string
    ? string
    : T[K] extends object
      ? DeepString<T[K]>
      : never;
};

type Translations = DeepString<typeof zh>;

const locales: Record<string, Translations> = {
  "zh-CN": zh as Translations,
  "en-US": en as Translations,
};

interface I18nState {
  locale: string;
  setLocale: (locale: string) => void;
  t: Translations;
}

/**
 * Returns the translation object for the current locale.
 * Usage: const { t } = useI18n(); →  t.message.empty
 */
export const useI18n = create<I18nState>(() => ({
  locale: "zh-CN",
  setLocale: (_locale: string) => {},
  t: zh,
}));

// Initialize immediately with zh-CN as default
useI18n.setState({
  locale: "zh-CN",
  t: locales["zh-CN"],
  setLocale: (locale: string) => {
    useI18n.setState({
      locale,
      t: locales[locale] || zh,
    });
  },
});

/** Simple string interpolation: replace {key} with values */
export function fmt(
  template: string,
  values: Record<string, string | number>,
): string {
  return template.replace(/\{(\w+)\}/g, (_, key) =>
    String(values[key] ?? `{${key}}`),
  );
}
