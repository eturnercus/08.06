import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../../store/appStore";
import { useAgentStream } from "../../hooks/useAgentStream";
import { useAgentOrchestration } from "../../hooks/useAgentOrchestration";
import { api } from "../../api/tauri";
import { ORCHESTRATION_STRATEGIES } from "../../constants/agents";

export function OrchestrationMonitor() {
  useAgentStream();
  useAgentOrchestration();
  const { t, i18n } = useTranslation();
  const lang = i18n.language === "ru" ? "ru" : "en";
  const { monitorEvents, clearMonitor, activeAgentTask } = useAppStore();

  const running = monitorEvents.filter((e) => e.status === "running").length;
  const rounds = useMemo(() => {
    const map = new Map<number, typeof monitorEvents>();
    for (const e of monitorEvents) {
      const r = e.round ?? 0;
      if (!map.has(r)) map.set(r, []);
      map.get(r)!.push(e);
    }
    return [...map.entries()].sort((a, b) => a[0] - b[0]);
  }, [monitorEvents]);

  const modeLabel = (id?: string) =>
    ORCHESTRATION_STRATEGIES.find((s) => s.id === id)?.[lang] ?? id ?? "—";

  const stopTeam = () => {
    api.stopAgentTeam(activeAgentTask?.taskId).catch(() => {});
  };

  const stopAgent = (agentId: string) => {
    if (!activeAgentTask) return;
    api.stopAgentMember(activeAgentTask.taskId, agentId).catch(() => {});
  };

  return (
    <div className="orch-monitor">
      <div className="orch-monitor-head">
        <div>
          <h3>{t("agents.monitorTitle")}</h3>
          <div className="orch-badges">
            <span className="badge badge-blue">{t("agents.monitorLive")}: {running}</span>
            {activeAgentTask && (
              <span className="badge badge-purple">{activeAgentTask.groupName}</span>
            )}
          </div>
        </div>
        <div className="orch-monitor-actions">
          {activeAgentTask && (
            <button type="button" className="m3-outlined-btn danger" onClick={stopTeam}>
              {t("agents.stopTeam")}
            </button>
          )}
          <button type="button" className="m3-outlined-btn" onClick={clearMonitor}>
            {t("agents.clearMonitor")}
          </button>
        </div>
      </div>

      <p className="field-hint">{t("agents.monitorDesc")}</p>

      {rounds.length > 0 && (
        <div className="orch-flow scroll-x">
          {rounds.map(([round, events]) => (
            <div key={round} className="orch-round-col">
              <div className="orch-round-title">
                {round === 0 ? t("agents.monitorTask") : `${t("agents.monitorRound")} ${round}`}
              </div>
              <div className="orch-round-lane">
                {events.map((e) => (
                  <div
                    key={e.id}
                    className={`orch-node ${e.status}${e.streaming ? " streaming" : ""}`}
                  >
                    <div className="orch-node-head">
                      <span className="orch-phase">{e.type}</span>
                      {e.agentName && <strong>{e.agentName}</strong>}
                      {e.modelId && <span className="mono orch-model">{e.modelId.slice(0, 20)}</span>}
                    </div>
                    <div className="orch-node-body">{e.message}</div>
                    {e.orchestrationMode && round > 0 && (
                      <div className="orch-mode">{modeLabel(e.orchestrationMode)}</div>
                    )}
                    {e.status === "running" && e.agentId && activeAgentTask && (
                      <button
                        type="button"
                        className="m3-tonal-btn sm danger"
                        onClick={() => stopAgent(e.agentId!)}
                      >
                        {t("agents.stopAgent")}
                      </button>
                    )}
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}

      <div className="monitor-timeline">
        {monitorEvents.length === 0 ? (
          <p className="orch-empty">{t("agents.noEvents")}</p>
        ) : (
          monitorEvents.map((e) => (
            <div
              key={e.id}
              className={`monitor-event ${e.status === "error" ? "error" : ""} ${e.streaming ? "monitor-live" : ""}`}
            >
              <div className="monitor-time">{new Date(e.timestamp).toLocaleTimeString()}</div>
              <div className="monitor-body">
                {e.agentName && (
                  <strong>
                    {e.agentName}
                    {e.agentId && <span className="mono"> · {e.agentId.slice(0, 8)}</span>}
                    {" · "}
                  </strong>
                )}
                <span className="mono">[{e.type}]</span> {e.message}
                {e.streaming && <span className="stream-cursor">▍</span>}
              </div>
              <span className={`m3-chip ${e.status === "running" ? "active" : ""}`}>{e.status}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
