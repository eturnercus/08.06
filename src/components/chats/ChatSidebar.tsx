import { useTranslation } from "react-i18next";
import { useAppStore } from "../../store/appStore";
import { useModels } from "../../hooks/useModels";

export function ChatSidebar() {
  const { t } = useTranslation();
  const { chats, activeChatId, setActiveChat, addChat, deleteChat, exportChat, settings } =
    useAppStore();
  const { models } = useModels();
  const groups = (settings?.agentGroups ?? []) as { id: string; name: string }[];

  const modelLabel = (id: string) =>
    models.find((m) => m.id === id)?.name?.slice(0, 18) ?? id.slice(0, 10);
  const groupLabel = (id: string) =>
    groups.find((g) => g.id === id)?.name ?? id.slice(0, 10);

  return (
    <aside className="m3-panel chat-sidebar">
      <div className="m3-panel-header">
        <h2>{t("nav.chats")}</h2>
        <button type="button" className="m3-tonal-btn chat-add-btn" onClick={() => addChat()}>
          +
        </button>
      </div>
      <div className="scroll-y chat-list">
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
                >
                  💾
                </button>
                <button
                  type="button"
                  className="chat-icon-btn danger"
                  title={t("chat.delete")}
                  onClick={() => {
                    if (confirm(t("chat.deleteConfirm"))) deleteChat(c.id);
                  }}
                >
                  ✕
                </button>
              </div>
            </div>
            <div className="chat-list-model mono">
              {c.agentGroupId ? `🧩 ${groupLabel(c.agentGroupId)}` : `🧠 ${modelLabel(c.modelId)}`}
            </div>
            <div className="perms">
              {c.permissions.stm && <span className="m3-chip sm">{t("chat.stm")}</span>}
              {c.permissions.ltm && <span className="m3-chip sm">{t("chat.ltm")}</span>}
              {c.permissions.internet && <span className="m3-chip sm active">🌐</span>}
              {c.permissions.microphone && <span className="m3-chip sm">🎤</span>}
              {c.permissions.camera && <span className="m3-chip sm">📷</span>}
            </div>
          </div>
        ))}
      </div>
    </aside>
  );
}
