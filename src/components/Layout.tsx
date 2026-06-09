import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import { api } from "../api/tauri";
import { ChatSidebar } from "./chats/ChatSidebar";
import i18n from "../i18n";

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
  const { t, i18n: i18nInst } = useTranslation();
  const { activeView, setActiveView, settings, setSettings } = useAppStore();
  const [sys, setSys] = useState<Record<string, unknown>>({});

  const switchLang = async (lang: "ru" | "en") => {
    i18n.changeLanguage(lang);
    localStorage.setItem("silenium-lang", lang);
    if (!settings) return;
    const updated = { ...settings, language: lang };
    await api.updateSettings(updated as never);
    setSettings(updated);
  };

  useEffect(() => {
    api.getSystemInfo().then(setSys).catch(() => {});
  }, []);

  const showChatPanel = activeView === "chats";

  return (
    <div className="m3-app">
      <nav className="m3-rail">
        <div className="m3-rail-logo">Si</div>
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
          <div className="m3-topbar-title-block">
            <h1>{t(`nav.${activeView}`)}</h1>
            <p className="m3-topbar-sub">{t(`nav.subtitle.${activeView}`)}</p>
          </div>
          <div className="lang-switch">
            <button type="button" className={i18nInst.language === "ru" ? "active" : ""} onClick={() => switchLang("ru")}>RU</button>
            <button type="button" className={i18nInst.language === "en" ? "active" : ""} onClick={() => switchLang("en")}>EN</button>
          </div>
          <span className="m3-chip">RAM {String(sys.ramLimitMb ?? "—")} MB</span>
          <span className="m3-chip">{String(sys.platform ?? "linux")}</span>
        </header>
        <div className="m3-content">{children}</div>
      </div>
    </div>
  );
}
