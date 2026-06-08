import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, NetworkLog } from "../api/tauri";
import { Tooltip } from "./ui/Tooltip";

export function NetworkView() {
  const { t } = useTranslation();
  const [logs, setLogs] = useState<NetworkLog[]>([]);
  const [searchQ, setSearchQ] = useState("");
  const [searching, setSearching] = useState(false);

  const refresh = () => api.getNetworkLogs().then(setLogs).catch(() => {});

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
        <p style={{ fontSize: 13, color: "var(--m3-on-surface-variant)", marginBottom: 12 }}>{t("network.ddgDesc")}</p>
        <div style={{ display: "flex", gap: 8 }}>
          <Tooltip text={t("network.ddgTip")}>
            <input className="m3-input" style={{ flex: 1 }} value={searchQ} onChange={(e) => setSearchQ(e.target.value)}
              placeholder={t("network.ddgPlaceholder")} onKeyDown={(e) => e.key === "Enter" && ddgSearch()} />
          </Tooltip>
          <button type="button" className="m3-filled-btn" onClick={ddgSearch} disabled={searching}>{t("network.ddgSearch")}</button>
        </div>
      </div>
      <div className="scroll monitor-timeline" style={{ flex: 1 }}>
        {logs.length === 0 ? (
          <p style={{ color: "var(--m3-outline)", textAlign: "center", padding: 40 }}>{t("network.noLogs")}</p>
        ) : logs.map((log) => (
          <div key={log.id} className={`monitor-event ${log.blocked ? "error" : ""}`}>
            <span className="m3-chip" style={{ fontSize: 10 }}>{log.blocked ? t("network.blocked") : t("network.allowed")}</span>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 12, fontWeight: 600 }}>{log.method} {log.url}</div>
              {log.blockReason && <div style={{ color: "var(--m3-error)", fontSize: 11 }}>{log.blockReason}</div>}
              {log.responsePreview && <pre style={{ fontSize: 11, color: "var(--m3-outline)", marginTop: 4, whiteSpace: "pre-wrap", maxHeight: 60, overflow: "hidden" }}>{log.responsePreview}</pre>}
            </div>
            <span style={{ fontSize: 11, color: "var(--m3-outline)" }}>{log.durationMs}ms</span>
          </div>
        ))}
      </div>
    </div>
  );
}
