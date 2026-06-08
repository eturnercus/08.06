import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import { api } from "../api/tauri";

export function ChatView() {
  const { t } = useTranslation();
  const { chats, activeChatId, addChat, addMessage } = useAppStore();
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);
  const [attachments, setAttachments] = useState<{ name: string; mimeType: string; sizeBytes: number }[]>([]);

  const chat = chats.find((c) => c.id === activeChatId);

  const handleSend = async () => {
    if (!input.trim() || !chat) return;
    setLoading(true);
    addMessage(chat.id, { role: "user", content: input });
    try {
      const resp = await api.sendChat({
        chatId: chat.id,
        modelId: chat.modelId,
        message: input,
        attachments: attachments.map((a) => ({ ...a, mimeType: a.mimeType })),
      });
      addMessage(chat.id, {
        role: "assistant",
        content: resp.content,
        tokens: resp.tokensUsed,
        latencyMs: resp.latencyMs,
      });
      setInput("");
      setAttachments([]);
    } catch (e) {
      addMessage(chat.id, { role: "assistant", content: `Error: ${e}` });
    }
    setLoading(false);
  };

  const handleFile = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files) return;
    const newAtt = Array.from(files).map((f) => ({
      name: f.name,
      mimeType: f.type || "application/octet-stream",
      sizeBytes: f.size,
    }));
    setAttachments((a) => [...a, ...newAtt]);
  };

  if (!chat) {
    return (
      <div className="chat-empty">
        <p>{t("chat.newChat")}</p>
        <button className="btn-primary" onClick={() => addChat()}>{t("chat.newChat")}</button>
      </div>
    );
  }

  return (
    <div className="chat-view">
      <div className="chat-header">
        <h2>{chat.title}</h2>
        <div className="chat-perms">
          {chat.permissions.stm && <span className="badge badge-blue">{t("chat.stm")}</span>}
          {chat.permissions.ltm && <span className="badge badge-blue">{t("chat.ltm")}</span>}
          {chat.permissions.internet && <span className="badge badge-green">{t("chat.internet")}</span>}
          {!chat.permissions.internet && <span className="badge badge-red">{t("network.blocked")}</span>}
        </div>
        <button className="btn-secondary" onClick={() => addChat()}>+</button>
      </div>
      <div className="chat-messages scroll-y">
        {chat.messages.map((m, i) => (
          <div key={i} className={`msg msg-${m.role}`}>
            <div className="msg-content">{m.content}</div>
            {m.tokens !== undefined && (
              <div className="msg-meta">
                {t("chat.tokens")}: {m.tokens} | {t("chat.latency")}: {m.latencyMs}ms
              </div>
            )}
          </div>
        ))}
        {loading && <div className="msg msg-assistant"><div className="msg-content">...</div></div>}
      </div>
      {attachments.length > 0 && (
        <div className="attachments">
          {attachments.map((a, i) => (
            <span key={i} className="badge badge-blue">{a.name}</span>
          ))}
        </div>
      )}
      <div className="chat-input-bar">
        <input ref={fileRef} type="file" multiple hidden onChange={handleFile}
          accept="image/*,audio/*,video/*,.pdf" />
        <button className="btn-secondary" onClick={() => fileRef.current?.click()} title={t("chat.attach")}>📎</button>
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={t("chat.placeholder")}
          rows={2}
          onKeyDown={(e) => { if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); handleSend(); } }}
        />
        <button className="btn-primary" onClick={handleSend} disabled={loading}>{t("chat.send")}</button>
      </div>
      <style>{`
        .chat-view { display: flex; flex-direction: column; height: 100%; }
        .chat-empty { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 16px; }
        .chat-header { display: flex; align-items: center; gap: 12px; padding: 12px 16px; border-bottom: 1px solid var(--border); }
        .chat-header h2 { flex: 1; font-size: 16px; }
        .chat-perms { display: flex; gap: 6px; flex-wrap: wrap; }
        .chat-messages { flex: 1; padding: 16px; display: flex; flex-direction: column; gap: 12px; }
        .msg { max-width: 85%; }
        .msg-user { align-self: flex-end; }
        .msg-assistant { align-self: flex-start; }
        .msg-content {
          padding: 10px 14px; border-radius: var(--radius); line-height: 1.5;
          white-space: pre-wrap; word-break: break-word;
        }
        .msg-user .msg-content { background: var(--accent); color: white; }
        .msg-assistant .msg-content { background: var(--bg3); border: 1px solid var(--border); }
        .msg-meta { font-size: 11px; color: var(--text2); margin-top: 4px; }
        .attachments { padding: 4px 16px; display: flex; gap: 6px; flex-wrap: wrap; }
        .chat-input-bar {
          display: flex; gap: 8px; padding: 12px 16px; border-top: 1px solid var(--border);
          align-items: flex-end;
        }
        .chat-input-bar textarea { flex: 1; resize: none; }
      `}</style>
    </div>
  );
}
