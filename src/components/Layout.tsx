import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";

const NAV = ["chats", "agents", "models", "memory", "network", "devices", "settings", "help"] as const;

export function Layout({ children }: { children: React.ReactNode }) {
  const { t } = useTranslation();
  const { activeView, setActiveView } = useAppStore();

  return (
    <div className="layout">
      <aside className="sidebar">
        <div className="sidebar-brand">
          <span className="brand-icon">NF</span>
          <div>
            <div className="brand-name">{t("app.name")}</div>
            <div className="brand-tag">{t("app.tagline")}</div>
          </div>
        </div>
        <nav className="sidebar-nav">
          {NAV.map((key) => (
            <button
              key={key}
              className={`nav-item ${activeView === key ? "active" : ""}`}
              onClick={() => setActiveView(key)}
            >
              {t(`nav.${key}`)}
            </button>
          ))}
        </nav>
      </aside>
      <main className="main-content">{children}</main>
      <style>{`
        .layout { display: flex; height: 100vh; overflow: hidden; }
        .sidebar {
          width: var(--sidebar-w); min-width: var(--sidebar-w);
          background: var(--bg2); border-right: 1px solid var(--border);
          display: flex; flex-direction: column; padding: 16px 12px;
        }
        .sidebar-brand { display: flex; gap: 10px; align-items: center; padding: 8px; margin-bottom: 16px; }
        .brand-icon {
          width: 36px; height: 36px; border-radius: 8px; background: var(--accent);
          display: flex; align-items: center; justify-content: center;
          font-weight: 800; font-size: 13px; color: white; flex-shrink: 0;
        }
        .brand-name { font-weight: 700; font-size: 15px; }
        .brand-tag { font-size: 10px; color: var(--text2); line-height: 1.3; }
        .sidebar-nav { display: flex; flex-direction: column; gap: 2px; flex: 1; overflow-y: auto; }
        .nav-item {
          text-align: left; padding: 9px 12px; border-radius: 8px;
          background: transparent; color: var(--text2); font-size: 13px;
          border: none; transition: background 0.1s;
        }
        .nav-item:hover { background: var(--bg3); color: var(--text); }
        .nav-item.active { background: #1e1b4b; color: var(--accent2); font-weight: 600; }
        .main-content { flex: 1; overflow: hidden; display: flex; flex-direction: column; }
      `}</style>
    </div>
  );
}
