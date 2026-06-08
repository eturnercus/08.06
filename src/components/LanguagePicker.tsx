import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import "../styles/welcome.css";

const LANGUAGES = [
  { code: "ru", flag: "🇷🇺", native: "Русский", sub: "Russian" },
  { code: "en", flag: "🇬🇧", native: "English", sub: "Английский" },
] as const;

export function LanguagePicker() {
  const { t, i18n } = useTranslation();
  const setPhase = useAppStore((s) => s.setPhase);

  const select = (lang: string) => {
    i18n.changeLanguage(lang);
    localStorage.setItem("neuroforge-lang", lang);
    setPhase("onboarding");
  };

  return (
    <div className="lang-screen">
      <div className="lang-bg" />
      <div className="lang-inner">
        <div className="lang-logo-block">
          <div className="lang-logo">NF</div>
          <h1>{t("app.name")}</h1>
          <p>{t("app.tagline")}</p>
        </div>

        <div className="lang-card">
          <h2>{t("language.title")}</h2>
          <p className="lang-sub">{t("language.subtitle")}</p>

          <div className="lang-list">
            {LANGUAGES.map((l) => (
              <button type="button" key={l.code} className="lang-item" onClick={() => select(l.code)}>
                <span className="lang-item-flag">{l.flag}</span>
                <div className="lang-item-text">
                  <div className="lang-item-native">{l.native}</div>
                  <div className="lang-item-sub">{l.sub}</div>
                </div>
                <span className="lang-item-arrow">→</span>
              </button>
            ))}
          </div>

          <div className="lang-feats">
            <span>{t("language.features.local")}</span>
            <span>{t("language.features.secure")}</span>
            <span>{t("language.features.innovative")}</span>
          </div>
        </div>

        <p className="lang-ver mono">v1.0.0</p>
      </div>
    </div>
  );
}
