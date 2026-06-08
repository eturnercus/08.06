import { useTranslation } from "react-i18next";

const SECTIONS = ["system", "network", "memory", "injection", "agents", "devices", "reset"] as const;

const FEATURE_KEYS = [
  "orchestration", "roles", "tools", "permissions", "resources", "groupSettings",
  "triggers", "monitor", "memoryLevels", "settings79", "onboarding9", "tooltips", "hf", "m3",
] as const;

export function HelpView() {
  const { t } = useTranslation();

  return (
    <div className="help-view scroll">
      <h2>{t("help.title")}</h2>
      <p className="help-intro">{t("help.intro")}</p>

      <div className="m3-card" style={{ marginBottom: 20 }}>
        <h3 style={{ fontSize: 15, marginBottom: 12 }}>✓ {t("help.featureMatrix")}</h3>
        <div className="help-feature-grid">
          {FEATURE_KEYS.map((key) => (
            <div key={key} className="help-feature-item">
              <span className="help-check">✓</span>
              <span>{t(`help.features.${key}`)}</span>
            </div>
          ))}
        </div>
      </div>

      {SECTIONS.map((key) => (
        <div key={key} className="m3-card help-section">
          <h3>{t(`settings.tabs.${key === "injection" ? "injection" : key === "reset" ? "advanced" : key === "system" ? "ram" : key}`)}</h3>
          <p>{t(`help.sections.${key}`)}</p>
        </div>
      ))}
    </div>
  );
}
