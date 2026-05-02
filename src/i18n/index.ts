import { createContext, useContext } from "react";
import { translations, type Lang, type Translations } from "./translations";

export type { Lang } from "./translations";

export const I18nContext = createContext<Translations>(translations.en);

export function getLang(language: string): Lang {
  return language === "zh" ? "zh" : "en";
}

export function useI18n() {
  return useContext(I18nContext);
}

export { translations };
