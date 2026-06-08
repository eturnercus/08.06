import { useTranslation } from "react-i18next";
import { useAppStore } from "../../store/appStore";

export function OrchestrationMonitor() {
  const { t } = useTranslation();
  const { monitorEvents, clearMonitor } = useAppStore();

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 12 }}>
        <h3 style={{ fontSize: 15 }}>{t("agents.monitorTitle")}</h3>
        <button type="button" className="m3-outlined-btn" onClick={clearMonitor}>{t("agents.clearMonitor")}</button>
      </div>
      <p style={{ fontSize: 13, color: "var(--m3-on-surface-variant)", marginBottom: 16 }}>{t("agents.monitorDesc")}</p>
      <div className="monitor-timeline">
        {monitorEvents.length === 0 ? (
          <p style={{ color: "var(--m3-outline)", textAlign: "center", padding: 40 }}>{t("agents.noEvents")}</p>
        ) : monitorEvents.map((e) => (
          <div key={e.id} className={`monitor-event ${e.status === "error" ? "error" : ""}`}>
            <div style={{ minWidth: 70, fontSize: 11, color: "var(--m3-outline)" }}>
              {new Date(e.timestamp).toLocaleTimeString()}
            </div>
            <div style={{ flex: 1 }}>
              {e.agentName && <strong>{e.agentName} · </strong>}
              <span style={{ color: "var(--m3-on-surface-variant)" }}>[{e.type}]</span> {e.message}
            </div>
            <span className="m3-chip" style={{ fontSize: 10 }}>{e.status}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
