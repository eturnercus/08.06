import { useTranslation } from "react-i18next";
import { useAppStore } from "../../store/appStore";
import { useModels } from "../../hooks/useModels";
import { ChatSettingsPanel } from "./ChatSettingsPanel";

export function ChatSidebar() {
  const { t } = useTranslation();
  const { chats, activeChatId, setActiveChat, addChat } = useAppStore();
  const { models } = useModels();

  const modelLabel = (id: string) => models.find((m) => m.id === id)?.name?.slice(0, 12) ?? id.slice(0, 8);

  return (
    <aside className="m3-panel chat-sidebar">
      <div className="m3-panel-header">
        <h2>{t("nav.chats")}</h2>
        <button type="button" className="m3-tonal-btn chat-add-btn" onClick={() => addChat()}>+</button>
      </div>
      <div className="scroll chat-list">
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
            <div className="chat-list-model mono">{modelLabel(c.modelId)}</div>
            <div className="perms">
              {c.permissions.stm && <span className="m3-chip sm">STM</span>}
              {c.permissions.ltm && <span className="m3-chip sm">LTM</span>}
              {c.permissions.internet && <span className="m3-chip sm active">🌐</span>}
              {c.permissions.microphone && <span className="m3-chip sm">🎤</span>}
              {c.permissions.camera && <span className="m3-chip sm">📷</span>}
            </div>
          </div>
        ))}
      </div>
      {activeChatId && <ChatSettingsPanel chatId={activeChatId} />}
    </aside>
  );
}
