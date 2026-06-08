import { useTranslation } from "react-i18next";

const SECTIONS = ["system", "network", "memory", "injection", "agents", "devices", "reset"] as const;

export function HelpView() {
  const { t } = useTranslation();

  return (
    <div className="help-view scroll-y">
      <h2>{t("help.title")}</h2>
      <p className="intro">{t("help.intro")}</p>
      {SECTIONS.map((key) => (
        <div key={key} className="help-section card">
          <h3>{t(`settings.tabs.${key === "injection" ? "injection" : key === "reset" ? "advanced" : key}`)}</h3>
          <p>{t(`help.sections.${key}`)}</p>
        </div>
      ))}
      <style>{`
        .help-view { padding: 16px 20px; height: 100%; }
        h2 { margin-bottom: 8px; }
        .intro { color: var(--text2); margin-bottom: 20px; line-height: 1.6; }
        .help-section { margin-bottom: 12px; }
        .help-section h3 { font-size: 14px; margin-bottom: 8px; color: var(--accent2); }
        .help-section p { font-size: 13px; line-height: 1.6; color: var(--text2); }
      `}</style>
    </div>
  );
}
