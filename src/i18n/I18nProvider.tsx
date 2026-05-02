import { useMemo, type ReactNode } from "react";
import { I18nContext, getLang, translations } from "./index";

interface Props {
  language: string;
  children: ReactNode;
}

export function I18nProvider({ language, children }: Props) {
  const t = useMemo(() => translations[getLang(language)], [language]);
  return <I18nContext.Provider value={t}>{children}</I18nContext.Provider>;
}
