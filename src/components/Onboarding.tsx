import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import { api } from "../api/tauri";

const STEPS = ["welcome", "step1", "step2", "step3", "step4", "step5"] as const;

export function Onboarding() {
  const { t } = useTranslation();
  const { onboardingStep, setOnboardingStep, setPhase, settings, setSettings } = useAppStore();

  const finish = async () => {
    if (settings) {
      const updated = { ...settings, firstRunCompleted: true, onboardingStep: STEPS.length };
      await api.updateSettings(updated as never);
      setSettings(updated);
    }
    setPhase("app");
  };

  const next = () => {
    if (onboardingStep >= STEPS.length - 1) finish();
    else setOnboardingStep(onboardingStep + 1);
  };

  return (
    <div className="onboarding-screen">
      <div className="onboarding-card card">
        <div className="step-dots">
          {STEPS.map((_, i) => (
            <span key={i} className={`dot ${i <= onboardingStep ? "active" : ""}`} />
          ))}
        </div>
        <h2>{t(`onboarding.${STEPS[onboardingStep]}`)}</h2>
        <div className="onboarding-actions">
          <button className="btn-secondary" onClick={finish}>{t("onboarding.skip")}</button>
          <button className="btn-primary" onClick={next}>
            {onboardingStep >= STEPS.length - 1 ? t("onboarding.finish") : t("onboarding.next")}
          </button>
        </div>
      </div>
      <style>{`
        .onboarding-screen {
          display: flex; align-items: center; justify-content: center; height: 100vh;
        }
        .onboarding-card { max-width: 520px; width: 90%; padding: 36px; text-align: center; }
        h2 { font-size: 20px; line-height: 1.5; margin: 24px 0 32px; min-height: 60px; }
        .step-dots { display: flex; gap: 8px; justify-content: center; }
        .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--border); }
        .dot.active { background: var(--accent); }
        .onboarding-actions { display: flex; gap: 12px; justify-content: center; }
      `}</style>
    </div>
  );
}
