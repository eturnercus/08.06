import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import { api, AgentTask } from "../api/tauri";

export function AgentsView() {
  const { t } = useTranslation();
  const { settings } = useAppStore();
  const [prompt, setPrompt] = useState("");
  const [task, setTask] = useState<AgentTask | null>(null);
  const [loading, setLoading] = useState(false);

  const groups = settings?.agentGroups || [];
  const [groupId, setGroupId] = useState(groups[0]?.id || "");

  const run = async () => {
    if (!groupId || !prompt) return;
    setLoading(true);
    try {
      const result = await api.runAgentTeam(groupId, prompt);
      setTask(result);
    } catch (e) {
      setTask({ id: "err", status: "error", prompt, rounds: [{ roundNumber: 1, messages: [{ agentName: "System", role: "error", content: String(e), usedInternet: false }] }] });
    }
    setLoading(false);
  };

  return (
    <div className="agents-view">
      <h2>{t("agents.title")}</h2>
      {groups.length === 0 ? (
        <p className="hint">Добавьте группу агентов в Настройках → Группы агентов</p>
      ) : (
        <>
          <select value={groupId} onChange={(e) => setGroupId(e.target.value)} style={{ marginBottom: 12, width: "100%", maxWidth: 400 }}>
            {groups.map((g) => <option key={g.id} value={g.id}>{g.name}</option>)}
          </select>
          <textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder={t("agents.prompt")}
            rows={3}
            style={{ width: "100%", maxWidth: 600, marginBottom: 12 }}
          />
          <button className="btn-primary" onClick={run} disabled={loading}>{t("agents.runTeam")}</button>
        </>
      )}
      {task && (
        <div className="task-results">
          <h3>{t("agents.results")} — {task.status}</h3>
          {task.rounds.map((r) => (
            <div key={r.roundNumber} className="card" style={{ marginTop: 12 }}>
              <strong>Round {r.roundNumber}</strong>
              {r.messages.map((m, i) => (
                <div key={i} className="agent-msg">
                  <span className="badge badge-blue">{m.agentName} / {m.role}</span>
                  {m.usedInternet && <span className="badge badge-green">🌐</span>}
                  <p>{m.content}</p>
                </div>
              ))}
            </div>
          ))}
        </div>
      )}
      <style>{`
        .agents-view { padding: 16px; overflow-y: auto; height: 100%; }
        .hint { color: var(--text2); margin-top: 12px; }
        .agent-msg { margin-top: 10px; padding-top: 10px; border-top: 1px solid var(--border); }
        .agent-msg p { margin-top: 6px; font-size: 13px; line-height: 1.5; white-space: pre-wrap; }
      `}</style>
    </div>
  );
}
