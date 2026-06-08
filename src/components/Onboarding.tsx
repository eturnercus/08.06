import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import { api } from "../api/tauri";
import "../styles/welcome.css";

const STEP_KEYS = ["welcome", "local", "isolation", "memory", "injection", "agents", "permissions", "innovation", "ready"] as const;

const STEP_META: Record<string, { icon: string; color: string; features: string[] }> = {
  welcome: { icon: "⚡", color: "#7c6cff", features: ["feat1", "feat2", "feat3"] },
  local: { icon: "🧠", color: "#22d3ee", features: ["feat1", "feat2", "feat3"] },
  isolation: { icon: "🛡️", color: "#34d399", features: ["feat1", "feat2", "feat3"] },
  memory: { icon: "💾", color: "#f472b6", features: ["feat1", "feat2", "feat3"] },
  injection: { icon: "✨", color: "#fbbf24", features: ["feat1", "feat2", "feat3"] },
  agents: { icon: "🤝", color: "#a78bfa", features: ["feat1", "feat2", "feat3"] },
  permissions: { icon: "🔐", color: "#34d399", features: ["feat1", "feat2", "feat3"] },
  innovation: { icon: "🔮", color: "#22d3ee", features: ["feat1", "feat2", "feat3"] },
  ready: { icon: "🚀", color: "#7c6cff", features: ["feat1", "feat2"] },
};

export function Onboarding() {
  const { t } = useTranslation();
  const { onboardingStep, setOnboardingStep, setPhase, settings, setSettings } = useAppStore();
  const stepKey = STEP_KEYS[onboardingStep] ?? "welcome";
  const meta = STEP_META[stepKey];
  const progress = ((onboardingStep + 1) / STEP_KEYS.length) * 100;

  const finish = async () => {
    if (settings) {
      const updated = { ...settings, firstRunCompleted: true, onboardingStep: STEP_KEYS.length };
      await api.updateSettings(updated as never);
      setSettings(updated);
    }
    setPhase("app");
  };

  const next = () => {
    if (onboardingStep >= STEP_KEYS.length - 1) finish();
    else setOnboardingStep(onboardingStep + 1);
  };

  const prev = () => {
    if (onboardingStep > 0) setOnboardingStep(onboardingStep - 1);
  };

  return (
    <div className="onb">
      <div className="mesh-bg"><div className="mesh-grid" /></div>

      <div className="onb-shell">
        <header className="onb-header">
          <div className="onb-brand">
            <span className="onb-logo">NF</span>
            <span>{t("app.name")}</span>
          </div>
          <div className="onb-progress-wrap">
            <div className="onb-step-label">
              {t("onboarding.stepOf", { current: onboardingStep + 1, total: STEP_KEYS.length })}
            </div>
            <div className="progress-bar"><div className="progress-fill" style={{ width: `${progress}%` }} /></div>
          </div>
          <button className="btn btn-ghost" onClick={finish}>{t("onboarding.skip")}</button>
        </header>

        <div className="onb-body card card-glow">
          <div className="onb-visual" style={{ "--step-color": meta.color } as React.CSSProperties}>
            <div className="onb-icon-ring">
              <span className="onb-icon">{meta.icon}</span>
            </div>
          </div>

          <div className="onb-content">
            <span className="badge badge-innovation">{t(`onboarding.tags.${stepKey}`)}</span>
            <h1>{t(`onboarding.${stepKey}.title`)}</h1>
            <p className="onb-desc">{t(`onboarding.${stepKey}.desc`)}</p>

            <ul className="onb-features">
              {meta.features.map((f) => (
                <li key={f}>
                  <span className="onb-check">✓</span>
                  {t(`onboarding.${stepKey}.${f}`)}
                </li>
              ))}
            </ul>

            {stepKey === "innovation" && (
              <div className="onb-innovation-preview">
                {["cognitive", "quantum", "holographic"].map((k) => (
                  <div key={k} className="onb-innov-chip">
                    <span>{t(`onboarding.innovation.chips.${k}`)}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        <footer className="onb-footer">
          <div className="onb-dots">
            {STEP_KEYS.map((_, i) => (
              <button
                key={i}
                className={`onb-dot ${i === onboardingStep ? "active" : ""} ${i < onboardingStep ? "done" : ""}`}
                onClick={() => setOnboardingStep(i)}
                aria-label={`Step ${i + 1}`}
              />
            ))}
          </div>
          <div className="onb-actions">
            {onboardingStep > 0 && (
              <button className="btn btn-secondary" onClick={prev}>{t("onboarding.back")}</button>
            )}
            <button className="btn btn-primary btn-lg" onClick={next}>
              {onboardingStep >= STEP_KEYS.length - 1 ? t("onboarding.finish") : t("onboarding.next")}
              <span>→</span>
            </button>
          </div>
        </footer>
      </div>

    </div>
  );
}
