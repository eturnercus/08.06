import { useTranslation } from "react-i18next";
import { useAppStore } from "../../store/appStore";
import { useAgentStream } from "../../hooks/useAgentStream";

export function OrchestrationMonitor() {
  useAgentStream();
  const { t } = useTranslation();
  const { monitorEvents, clearMonitor } = useAppStore();

  const running = monitorEvents.filter((e) => e.status === "running").length;
  const agents = new Set(monitorEvents.filter((e) => e.agentId).map((e) => e.agentId)).size;

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 12, alignItems: "center" }}>
        <div>
          <h3 style={{ fontSize: 15 }}>{t("agents.monitorTitle")}</h3>
          <div style={{ display: "flex", gap: 8, marginTop: 6 }}>
            <span className="badge badge-blue">{t("agents.monitorLive")}: {running}</span>
            <span className="badge badge-purple">{t("agents.monitorAgents")}: {agents}</span>
          </div>
        </div>
        <button type="button" className="m3-outlined-btn" onClick={clearMonitor}>{t("agents.clearMonitor")}</button>
      </div>
      <p style={{ fontSize: 13, color: "var(--m3-on-surface-variant)", marginBottom: 16 }}>{t("agents.monitorDesc")}</p>
      <div className="monitor-timeline">
        {monitorEvents.length === 0 ? (
          <p style={{ color: "var(--m3-outline)", textAlign: "center", padding: 40 }}>{t("agents.noEvents")}</p>
        ) : monitorEvents.map((e) => (
          <div
            key={e.id}
            className={`monitor-event ${e.status === "error" ? "error" : ""} ${e.streaming ? "monitor-live" : ""}`}
          >
            <div style={{ minWidth: 70, fontSize: 11, color: "var(--m3-outline)" }}>
              {new Date(e.timestamp).toLocaleTimeString()}
            </div>
            <div style={{ flex: 1, minWidth: 0 }}>
              {e.agentName && (
                <strong style={{ color: e.streaming ? "var(--accent-bright)" : undefined }}>
                  {e.agentName}
                  {e.agentId && <span className="mono" style={{ fontWeight: 400, opacity: 0.7 }}> · {e.agentId.slice(0, 8)}</span>}
                  {" · "}
                </strong>
              )}
              <span style={{ color: "var(--m3-on-surface-variant)" }}>[{e.type}]</span>{" "}
              {e.message}
              {e.streaming && <span className="stream-cursor">▍</span>}
            </div>
            <span className={`m3-chip ${e.status === "running" ? "active" : ""}`} style={{ fontSize: 10 }}>
              {e.status}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
