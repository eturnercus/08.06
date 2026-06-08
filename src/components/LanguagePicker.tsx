import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";

export function LanguagePicker() {
  const { t, i18n } = useTranslation();
  const setPhase = useAppStore((s) => s.setPhase);

  const select = (lang: string) => {
    i18n.changeLanguage(lang);
    localStorage.setItem("neuroforge-lang", lang);
    setPhase("onboarding");
  };

  return (
    <div className="language-screen">
      <div className="language-card card">
        <div className="language-logo">NF</div>
        <h1>{t("language.title")}</h1>
        <p className="subtitle">{t("language.subtitle")}</p>
        <div className="language-buttons">
          <button className="btn-lang" onClick={() => select("ru")}>
            <span className="flag">🇷🇺</span>
            {t("language.ru")}
          </button>
          <button className="btn-lang" onClick={() => select("en")}>
            <span className="flag">🇬🇧</span>
            {t("language.en")}
          </button>
        </div>
      </div>
      <style>{`
        .language-screen {
          display: flex; align-items: center; justify-content: center;
          height: 100vh; background: radial-gradient(ellipse at 50% 0%, #1e1b4b 0%, var(--bg) 60%);
        }
        .language-card { text-align: center; max-width: 420px; width: 90%; padding: 40px 32px; }
        .language-logo {
          width: 64px; height: 64px; border-radius: 16px; background: var(--accent);
          display: flex; align-items: center; justify-content: center;
          font-size: 24px; font-weight: 800; color: white; margin: 0 auto 20px;
        }
        h1 { font-size: 22px; margin-bottom: 8px; }
        .subtitle { color: var(--text2); margin-bottom: 28px; }
        .language-buttons { display: flex; flex-direction: column; gap: 12px; }
        .btn-lang {
          display: flex; align-items: center; gap: 12px; padding: 14px 20px;
          background: var(--bg3); border: 1px solid var(--border); border-radius: var(--radius);
          color: var(--text); font-size: 16px; transition: border-color 0.15s;
        }
        .btn-lang:hover { border-color: var(--accent); }
        .flag { font-size: 24px; }
      `}</style>
    </div>
  );
}
