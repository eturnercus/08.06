import { useTranslation } from "react-i18next";
import { useAppStore } from "../../store/appStore";
import { ChatPermissionsModal } from "./ChatPermissionsModal";

export function ChatSidebar() {
  const { t, i18n } = useTranslation();
  const { chats, activeChatId, setActiveChat, addChat } = useAppStore();
  const lang = i18n.language === "ru" ? "ru" : "en";

  return (
    <aside className="m3-panel" style={{ width: "var(--chat-panel-w)" }}>
      <div className="m3-panel-header">
        <h2>{t("nav.chats")}</h2>
        <button type="button" className="m3-tonal-btn" style={{ padding: "6px 14px", fontSize: 12 }} onClick={() => addChat()}>+</button>
      </div>
      <div className="scroll" style={{ flex: 1 }}>
        {chats.map((c) => (
          <div
            key={c.id}
            className={`chat-list-item ${c.id === activeChatId ? "active" : ""}`}
            onClick={() => setActiveChat(c.id)}
            onKeyDown={(e) => e.key === "Enter" && setActiveChat(c.id)}
            role="button"
            tabIndex={0}
          >
            <div className="title">{c.title}</div>
            <div className="perms">
              {c.permissions.stm && <span className="m3-chip" style={{ padding: "2px 8px", fontSize: 10 }}>STM</span>}
              {c.permissions.ltm && <span className="m3-chip" style={{ padding: "2px 8px", fontSize: 10 }}>LTM</span>}
              {c.permissions.internet && <span className="m3-chip active" style={{ padding: "2px 8px", fontSize: 10 }}>🌐</span>}
              {c.agentGroupId && <span className="m3-chip" style={{ padding: "2px 8px", fontSize: 10 }}>🤝</span>}
            </div>
          </div>
        ))}
      </div>
      {activeChatId && <ChatPermissionsModal chatId={activeChatId} />}
    </aside>
  );
}
