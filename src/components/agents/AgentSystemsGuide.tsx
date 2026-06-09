import { useState } from "react";
import { useTranslation } from "react-i18next";
import { ORCHESTRATION_STRATEGIES } from "../../constants/agents";

export function AgentSystemsGuide() {
  const { t, i18n } = useTranslation();
  const lang = i18n.language === "ru" ? "ru" : "en";
  const [open, setOpen] = useState(false);

  return (
    <div className="agent-guide">
      <button
        type="button"
        className={`m3-tonal-btn agent-guide-toggle${open ? " active" : ""}`}
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
      >
        {open ? "▾" : "▸"} {t("agents.guide.title")}
      </button>
      {open && (
        <div className="agent-guide-panel scroll-y">
          <p className="agent-guide-lead">{t("agents.guide.lead")}</p>
          <section>
            <h4>{t("agents.guide.orchestrationTitle")}</h4>
            <p className="field-hint">{t("agents.tip.orchestration")}</p>
            <ul className="agent-guide-list">
              {ORCHESTRATION_STRATEGIES.map((s) => (
                <li key={s.id}>
                  <strong>{s[lang]}</strong>
                  <span className="mono"> · {s.id}</span>
                </li>
              ))}
            </ul>
          </section>
          <section>
            <h4>{t("agents.guide.conflictTitle")}</h4>
            <p className="field-hint">{t("agents.tip.conflict")}</p>
          </section>
          <section>
            <h4>{t("agents.guide.chatTitle")}</h4>
            <p>{t("agents.guide.chatBody")}</p>
          </section>
          <section>
            <h4>{t("agents.guide.permsTitle")}</h4>
            <p>{t("agents.guide.permsBody")}</p>
          </section>
        </div>
      )}
    </div>
  );
}
