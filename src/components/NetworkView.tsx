import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, NetworkLog } from "../api/tauri";
import { Tooltip } from "./ui/Tooltip";
import { isTauri } from "../api/browserFallback";

export function NetworkView() {
  const { t } = useTranslation();
  const [logs, setLogs] = useState<NetworkLog[]>([]);
  const [audit, setAudit] = useState<string[]>([]);
  const [tab, setTab] = useState<"http" | "audit">("http");
  const [searchQ, setSearchQ] = useState("");
  const [searching, setSearching] = useState(false);

  const refresh = () => {
    api.getNetworkLogs().then(setLogs).catch(() => {});
    if (isTauri()) api.getAuditLogs(150).then(setAudit).catch(() => {});
  };

  useEffect(() => {
    refresh();
    const iv = setInterval(refresh, 3000);
    return () => clearInterval(iv);
  }, []);

  const ddgSearch = async () => {
    if (!searchQ.trim()) return;
    setSearching(true);
    try {
      await api.webSearch(searchQ);
    } catch { /* logged */ }
    refresh();
    setSearching(false);
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div style={{ padding: 16, borderBottom: "1px solid var(--m3-outline-variant)" }}>
        <h3 style={{ marginBottom: 8 }}>{t("network.title")}</h3>
        <div className="m3-tabs" style={{ marginBottom: 12 }}>
          <button type="button" className={`m3-tab ${tab === "http" ? "active" : ""}`} onClick={() => setTab("http")}>
            HTTP / API
          </button>
          <button type="button" className={`m3-tab ${tab === "audit" ? "active" : ""}`} onClick={() => setTab("audit")}>
            {t("network.audit")}
          </button>
        </div>
        {tab === "http" && (
          <>
            <p style={{ fontSize: 13, color: "var(--m3-on-surface-variant)", marginBottom: 12 }}>{t("network.ddgDesc")}</p>
            <div style={{ display: "flex", gap: 8 }}>
              <Tooltip text={t("network.ddgTip")}>
                <input className="m3-input" style={{ flex: 1 }} value={searchQ} onChange={(e) => setSearchQ(e.target.value)}
                  placeholder={t("network.ddgPlaceholder")} onKeyDown={(e) => e.key === "Enter" && ddgSearch()} />
              </Tooltip>
              <button type="button" className="m3-filled-btn" onClick={ddgSearch} disabled={searching}>{t("network.ddgSearch")}</button>
            </div>
          </>
        )}
      </div>
      <div className="scroll monitor-timeline" style={{ flex: 1 }}>
        {tab === "http" ? (
          logs.length === 0 ? (
            <p style={{ color: "var(--m3-outline)", textAlign: "center", padding: 40 }}>{t("network.noLogs")}</p>
          ) : logs.map((log) => (
            <div key={log.id} className={`monitor-event ${log.blocked ? "error" : ""}`}>
              <span className="m3-chip" style={{ fontSize: 10 }}>{log.blocked ? t("network.blocked") : t("network.allowed")}</span>
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ fontSize: 11, color: "var(--m3-outline)", marginBottom: 2 }}>
                  {log.agentId && <span>🧩 {log.agentId} · </span>}
                  {log.chatId && <span>💬 {log.chatId.slice(0, 12)} · </span>}
                  {log.method}
                </div>
                <div style={{ fontSize: 12, fontWeight: 600 }}>{log.url}</div>
                {log.blockReason && <div style={{ color: "var(--m3-error)", fontSize: 11 }}>{log.blockReason}</div>}
                {log.responsePreview && (
                  <pre style={{ fontSize: 11, color: "var(--m3-outline)", marginTop: 4, whiteSpace: "pre-wrap", maxHeight: 80, overflow: "hidden" }}>
                    {log.responsePreview}
                  </pre>
                )}
              </div>
              <span style={{ fontSize: 11, color: "var(--m3-outline)" }}>{log.durationMs}ms</span>
            </div>
          ))
        ) : audit.length === 0 ? (
          <p style={{ color: "var(--m3-outline)", textAlign: "center", padding: 40 }}>{t("network.noAudit")}</p>
        ) : (
          audit.map((line, i) => (
            <div key={i} className="monitor-event">
              <pre style={{ fontSize: 11, whiteSpace: "pre-wrap", margin: 0, flex: 1 }}>{line}</pre>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
