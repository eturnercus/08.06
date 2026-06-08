import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import { api } from "../api/tauri";
import { ChatSidebar } from "./chats/ChatSidebar";

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
  }, []);

  const showChatPanel = activeView === "chats";

  return (
    <div className="m3-app">
      <nav className="m3-rail">
        <div className="m3-rail-logo">NF</div>
        {NAV.map(({ key, icon }) => (
          <button
            key={key}
            type="button"
            className={`m3-rail-btn ${activeView === key ? "active" : ""}`}
            onClick={() => setActiveView(key)}
            title={t(`nav.${key}`)}
          >
            <span className="m3-rail-icon">{icon}</span>
            <span className="m3-rail-label">{t(`nav.${key}`)}</span>
          </button>
        ))}
      </nav>

      {showChatPanel && <ChatSidebar />}

      <div className="m3-main">
        <header className="m3-topbar">
          <h1>{t(`nav.${activeView}`)}</h1>
          <span className="m3-chip">RAM {String(sys.ramLimitMb ?? "—")} MB</span>
          <span className="m3-chip">{String(sys.platform ?? "linux")}</span>
        </header>
        <div className="m3-content">{children}</div>
      </div>
    </div>
  );
}
