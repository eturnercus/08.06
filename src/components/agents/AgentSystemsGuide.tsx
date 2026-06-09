import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  AGENT_ROLES,
  AGENT_TOOLS,
  ORCHESTRATION_STRATEGIES,
  CONFLICT_MODES,
  TRIGGER_CONDITIONS,
} from "../../constants/agents";
import { ROLE_DEFAULT_PROMPTS, ROLE_DEFAULT_TOKENS } from "../../constants/rolePrompts";
import type { AgentMember } from "../../api/tauri";

type Props = {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  selectedMember?: AgentMember & { systemPromptCustomized?: boolean };
  onApplyRole?: (roleId: string) => void;
};

export function AgentSystemsGuide({ open: controlledOpen, onOpenChange, selectedMember, onApplyRole }: Props) {
  const { t, i18n } = useTranslation();
  const lang = i18n.language === "ru" ? "ru" : "en";
  const [internalOpen, setInternalOpen] = useState(false);
  const open = controlledOpen ?? internalOpen;
  const setOpen = onOpenChange ?? setInternalOpen;

  return (
    <div className="agent-guide">
      <button
        type="button"
        className={`m3-tonal-btn agent-guide-toggle${open ? " active" : ""}`}
        onClick={() => setOpen(!open)}
        aria-expanded={open}
      >
        ? {t("agents.guide.title")}
      </button>
      {open && (
        <div className="agent-guide-panel scroll-y">
          <p className="agent-guide-lead">{t("agents.guide.lead")}</p>

          <section>
            <h4>{t("agents.guide.rolesTitle")}</h4>
            <p className="field-hint">{t("agents.guide.rolesHint")}</p>
            <ul className="agent-guide-list">
              {AGENT_ROLES.map((r) => (
                <li key={r.id} className="agent-guide-role-item">
                  <div>
                    <strong>{r[lang]}</strong>
                    <span className="mono"> · {r.id}</span>
                    <p className="agent-guide-role-desc">{ROLE_DEFAULT_PROMPTS[r.id]?.[lang]}</p>
                    <span className="field-hint">{t("agents.guide.suggestedTokens")}: {ROLE_DEFAULT_TOKENS[r.id] ?? 2048}</span>
                  </div>
                  {onApplyRole && selectedMember && (
                    <button type="button" className="m3-tonal-btn sm" onClick={() => onApplyRole(r.id)}>
                      {t("agents.guide.applyRole")}
                    </button>
                  )}
                </li>
              ))}
            </ul>
          </section>

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
            <h4>{t("agents.guide.toolsTitle")}</h4>
            <p className="field-hint">{t("agents.guide.toolsHint")}</p>
            <div className="agent-guide-chips">
              {AGENT_TOOLS.map((tool) => (
                <span key={tool.id} className="m3-chip">{tool[lang]}</span>
              ))}
            </div>
          </section>

          <section>
            <h4>{t("agents.guide.triggersTitle")}</h4>
            <ul className="agent-guide-list">
              {TRIGGER_CONDITIONS.map((tr) => (
                <li key={tr.id}><strong>{tr[lang]}</strong><span className="mono"> · {tr.id}</span></li>
              ))}
            </ul>
          </section>

          <section>
            <h4>{t("agents.guide.conflictTitle")}</h4>
            <p className="field-hint">{t("agents.tip.conflict")}</p>
            <ul className="agent-guide-list">
              {CONFLICT_MODES.map((c) => (
                <li key={c.id}><strong>{c[lang]}</strong><span className="mono"> · {c.id}</span></li>
              ))}
            </ul>
          </section>

          <section>
            <h4>{t("agents.guide.chatTitle")}</h4>
            <p>{t("agents.guide.chatBody")}</p>
          </section>

          <section>
            <h4>{t("agents.guide.workspaceTitle")}</h4>
            <p>{t("agents.guide.workspaceBody")}</p>
          </section>

          <section>
            <h4>{t("agents.guide.permsTitle")}</h4>
            <p>{t("agents.guide.permsBody")}</p>
          </section>

          <section>
            <h4>{t("agents.guide.multiModelTitle")}</h4>
            <p>{t("agents.guide.multiModelBody")}</p>
          </section>
        </div>
      )}
    </div>
  );
}
