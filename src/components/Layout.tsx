import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import { api } from "../api/tauri";

const NAV: { key: string; icon: string }[] = [
  { key: "chats", icon: "💬" },
  { key: "agents", icon: "🤝" },
  { key: "models", icon: "🧠" },
  { key: "memory", icon: "💾" },
  { key: "network", icon: "🌐" },
  { key: "devices", icon: "📷" },
  { key: "settings", icon: "⚙️" },
  { key: "help", icon: "❓" },
];

export function Layout({ children }: { children: React.ReactNode }) {
  const { t } = useTranslation();
  const { activeView, setActiveView } = useAppStore();
  const [sys, setSys] = useState<Record<string, unknown>>({});

  useEffect(() => {
    api.getSystemInfo().then(setSys).catch(() => {});
    const iv = setInterval(() => api.getSystemInfo().then(setSys).catch(() => {}), 10000);
    return () => clearInterval(iv);
  }, []);

  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="sidebar-top">
          <div className="sidebar-brand">
            <div className="sb-logo">NF</div>
            {!false && (
              <div className="sb-brand-text">
                <span className="sb-name">{t("app.name")}</span>
                <span className="sb-ver mono">v1.0</span>
              </div>
            )}
          </div>
        </div>

        <nav className="sidebar-nav">
          {NAV.map(({ key, icon }) => (
            <button
              key={key}
              className={`nav-item ${activeView === key ? "active" : ""}`}
              onClick={() => setActiveView(key)}
            >
              <span className="nav-icon">{icon}</span>
              <span className="nav-label">{t(`nav.${key}`)}</span>
              {key === "settings" && <span className="nav-badge">150+</span>}
            </button>
          ))}
        </nav>

        <div className="sidebar-stats card">
          <div className="stat-row">
            <span>RAM</span>
            <span className="mono">{String(sys.ramLimitMb ?? "—")} MB</span>
          </div>
          <div className="stat-row">
            <span>CPU</span>
            <span className="mono">{Array.isArray(sys.cpuCores) ? (sys.cpuCores as number[]).length : "—"} cores</span>
          </div>
          <div className="stat-bar">
            <div className="stat-bar-fill" style={{ width: "42%" }} />
          </div>
        </div>
      </aside>

      <div className="main-wrap">
        <header className="topbar">
          <h1 className="topbar-title">{t(`nav.${activeView}`)}</h1>
          <div className="topbar-status">
            <span className="badge badge-green">● {t("layout.online")}</span>
            <span className="badge badge-blue mono">{String(sys.platform ?? "linux")}</span>
          </div>
        </header>
        <main className="main-content">{children}</main>
      </div>

      <style>{`
        .shell { display: flex; height: 100vh; background: var(--bg); }
        .sidebar {
          width: var(--sidebar-w); min-width: var(--sidebar-w);
          background: var(--bg-elevated); border-right: 1px solid var(--border);
          display: flex; flex-direction: column; padding: 16px 12px;
        }
        .sidebar-brand { display: flex; align-items: center; gap: 12px; padding: 4px 8px 16px; }
        .sb-logo {
          width: 40px; height: 40px; border-radius: 12px; flex-shrink: 0;
          background: linear-gradient(135deg, var(--accent), var(--accent-2));
          display: flex; align-items: center; justify-content: center;
          font-weight: 800; font-size: 14px; color: white;
        }
        .sb-brand-text { display: flex; flex-direction: column; }
        .sb-name { font-weight: 700; font-size: 15px; }
        .sb-ver { font-size: 10px; color: var(--text-muted); }
        .sidebar-nav { flex: 1; display: flex; flex-direction: column; gap: 2px; overflow-y: auto; }
        .nav-item {
          display: flex; align-items: center; gap: 10px;
          padding: 10px 12px; border-radius: var(--radius-sm);
          background: transparent; color: var(--text-secondary);
          font-size: 13px; font-weight: 500; border: none; text-align: left;
          transition: background 0.15s, color 0.15s;
        }
        .nav-item:hover { background: var(--bg-hover); color: var(--text); }
        .nav-item.active {
          background: linear-gradient(135deg, rgba(124,108,255,0.15), rgba(124,108,255,0.05));
          color: var(--accent-bright); font-weight: 600;
          border: 1px solid rgba(124,108,255,0.2);
        }
        .nav-icon { font-size: 16px; width: 22px; text-align: center; }
        .nav-label { flex: 1; }
        .nav-badge {
          font-size: 9px; padding: 2px 6px; border-radius: 99px;
          background: rgba(124,108,255,0.2); color: var(--accent-bright); font-weight: 700;
        }
        .sidebar-stats { padding: 12px; margin-top: 8px; }
        .stat-row { display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted); margin-bottom: 4px; }
        .stat-bar { height: 3px; background: var(--bg-surface); border-radius: 99px; margin-top: 8px; overflow: hidden; }
        .stat-bar-fill { height: 100%; background: linear-gradient(90deg, var(--accent), var(--accent-2)); border-radius: 99px; }
        .main-wrap { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
        .topbar {
          display: flex; align-items: center; justify-content: space-between;
          padding: 14px 24px; border-bottom: 1px solid var(--border);
          background: var(--bg-elevated);
        }
        .topbar-title { font-size: 18px; font-weight: 700; }
        .topbar-status { display: flex; gap: 8px; }
        .main-content { flex: 1; overflow: hidden; }
      `}</style>
    </div>
  );
}
