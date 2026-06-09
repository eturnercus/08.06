import { useTranslation } from "react-i18next";
import { useAppStore } from "../../store/appStore";
import { useModels } from "../../hooks/useModels";
import { ChatSettingsPanel } from "./ChatSettingsPanel";

export function ChatSidebar() {
  const { t } = useTranslation();
  const { chats, activeChatId, setActiveChat, addChat, deleteChat, exportChat } = useAppStore();
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
            <div className="chat-list-item-head">
              <div className="title">{c.title}</div>
              <div className="chat-list-actions" onClick={(e) => e.stopPropagation()}>
                <button
                  type="button"
                  className="chat-icon-btn"
                  title={t("chat.export")}
                  onClick={() => {
                    const blob = new Blob([exportChat(c.id)], { type: "application/json" });
                    const a = document.createElement("a");
                    a.href = URL.createObjectURL(blob);
                    a.download = `${c.title.replace(/\s+/g, "_")}.json`;
                    a.click();
                  }}
                >💾</button>
                <button
                  type="button"
                  className="chat-icon-btn danger"
                  title={t("chat.delete")}
                  onClick={() => { if (confirm(t("chat.deleteConfirm"))) deleteChat(c.id); }}
                >✕</button>
              </div>
            </div>
            <div className="chat-list-model mono">
              {c.agentGroupId ? `🧩 ${c.agentGroupId.slice(0, 10)}` : modelLabel(c.modelId)}
            </div>
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
