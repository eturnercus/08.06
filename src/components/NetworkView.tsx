import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, NetworkLog } from "../api/tauri";

export function NetworkView() {
  const { t } = useTranslation();
  const [logs, setLogs] = useState<NetworkLog[]>([]);

  const refresh = () => api.getNetworkLogs().then(setLogs).catch(() => {});

  useEffect(() => { refresh(); const iv = setInterval(refresh, 3000); return () => clearInterval(iv); }, []);

  const testFetch = async () => {
    try {
      await api.agentFetch("https://huggingface.co/api/models?limit=1");
    } catch { /* expected if blocked */ }
    refresh();
  };

  return (
    <div className="network-view">
      <div className="network-header">
        <h2>{t("network.title")}</h2>
        <button className="btn-primary" onClick={testFetch}>{t("network.fetch")}</button>
      </div>
      <div className="logs scroll-y">
        {logs.length === 0 ? (
          <p className="empty">{t("network.noLogs")}</p>
        ) : logs.map((log) => (
          <div key={log.id} className={`log-card card ${log.blocked ? "blocked" : ""}`}>
            <div className="log-top">
              <span className={`badge ${log.blocked ? "badge-red" : "badge-green"}`}>
                {log.blocked ? t("network.blocked") : t("network.allowed")}
              </span>
              <span className="method">{log.method}</span>
              <span className="duration">{log.durationMs}ms</span>
            </div>
            <div className="url">{log.url}</div>
            {log.blockReason && <div className="reason">{log.blockReason}</div>}
            {log.status && <div className="status">HTTP {log.status}</div>}
            {log.responsePreview && <pre className="preview">{log.responsePreview.slice(0, 300)}</pre>}
          </div>
        ))}
      </div>
      <style>{`
        .network-view { padding: 16px; display: flex; flex-direction: column; height: 100%; }
        .network-header { display: flex; align-items: center; gap: 12px; margin-bottom: 16px; }
        .network-header h2 { flex: 1; }
        .logs { flex: 1; display: flex; flex-direction: column; gap: 8px; }
        .log-card.blocked { border-color: var(--danger); }
        .log-top { display: flex; gap: 8px; align-items: center; margin-bottom: 6px; }
        .method { font-weight: 600; font-size: 12px; }
        .duration { color: var(--text2); font-size: 12px; margin-left: auto; }
        .url { font-size: 13px; word-break: break-all; margin-bottom: 4px; }
        .reason { color: var(--danger); font-size: 12px; }
        .status { color: var(--success); font-size: 12px; }
        .preview { font-size: 11px; color: var(--text2); margin-top: 6px; white-space: pre-wrap; max-height: 80px; overflow: hidden; }
        .empty { color: var(--text2); }
      `}</style>
    </div>
  );
}
