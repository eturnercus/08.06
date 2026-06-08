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
        chatId: chat.id, modelId: chat.modelId, message: input,
        attachments: attachments.map((a) => ({ ...a, mimeType: a.mimeType })),
      });
      addMessage(chat.id, { role: "assistant", content: resp.content, tokens: resp.tokensUsed, latencyMs: resp.latencyMs });
      setInput("");
      setAttachments([]);
    } catch (e) {
      addMessage(chat.id, { role: "assistant", content: `Error: ${e}` });
    }
    setLoading(false);
  };

  if (!chat) {
    return (
      <div className="chat-empty">
        <div className="chat-empty-icon">💬</div>
        <h2>{t("chat.newChat")}</h2>
        <button className="btn btn-primary btn-lg" onClick={() => addChat()}>{t("chat.newChat")}</button>
      </div>
    );
  }

  return (
    <div className="chat">
      <div className="chat-toolbar">
        <div className="chat-title-wrap">
          <h2>{chat.title}</h2>
          <div className="chat-badges">
            {chat.permissions.stm && <span className="badge badge-blue">{t("chat.stm")}</span>}
            {chat.permissions.ltm && <span className="badge badge-cyan">{t("chat.ltm")}</span>}
            {chat.permissions.internet
              ? <span className="badge badge-green">{t("chat.internet")}</span>
              : <span className="badge badge-red">{t("chat.offline")}</span>}
          </div>
        </div>
        <button className="btn btn-secondary" onClick={() => addChat()}>+ {t("chat.newChat")}</button>
      </div>

      <div className="chat-messages scroll-y">
        {chat.messages.length === 0 && (
          <div className="chat-welcome">
            <span className="chat-welcome-icon">🧠</span>
            <p>NeuroForge</p>
          </div>
        )}
        {chat.messages.map((m, i) => (
          <div key={i} className={`bubble-row ${m.role}`}>
            <div className="bubble-avatar">{m.role === "user" ? "👤" : "🤖"}</div>
            <div className="bubble">
              <div className="bubble-text">{m.content}</div>
              {m.tokens !== undefined && (
                <div className="bubble-meta mono">
                  {t("chat.tokens")}: {m.tokens} · {t("chat.latency")}: {m.latencyMs}ms
                </div>
              )}
            </div>
          </div>
        ))}
        {loading && (
          <div className="bubble-row assistant">
            <div className="bubble-avatar">🤖</div>
            <div className="bubble typing"><span /><span /><span /></div>
          </div>
        )}
      </div>

      {attachments.length > 0 && (
        <div className="chat-att">
          {attachments.map((a, i) => <span key={i} className="badge badge-blue">{a.name}</span>)}
        </div>
      )}

      <div className="chat-composer">
        <input ref={fileRef} type="file" multiple hidden onChange={(e) => {
          const files = e.target.files;
          if (!files) return;
          setAttachments((prev) => [...prev, ...Array.from(files).map((f) => ({
            name: f.name, mimeType: f.type || "application/octet-stream", sizeBytes: f.size,
          }))]);
        }} accept="image/*,audio/*,video/*,.pdf" />
        <button className="composer-btn" onClick={() => fileRef.current?.click()} title={t("chat.attach")}>📎</button>
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={t("chat.placeholder")}
          rows={1}
          onKeyDown={(e) => { if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); handleSend(); } }}
        />
        <button className="btn btn-primary composer-send" onClick={handleSend} disabled={loading || !input.trim()}>
          {t("chat.send")}
        </button>
      </div>

      <style>{`
        .chat { display: flex; flex-direction: column; height: 100%; }
        .chat-empty { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 16px; }
        .chat-empty-icon { font-size: 48px; opacity: 0.5; }
        .chat-toolbar { display: flex; align-items: center; gap: 16px; padding: 12px 24px; border-bottom: 1px solid var(--border); }
        .chat-title-wrap { flex: 1; }
        .chat-title-wrap h2 { font-size: 15px; font-weight: 600; margin-bottom: 4px; }
        .chat-badges { display: flex; gap: 6px; flex-wrap: wrap; }
        .chat-messages { flex: 1; padding: 20px 24px; display: flex; flex-direction: column; gap: 16px; }
        .chat-welcome { text-align: center; margin: auto; color: var(--text-muted); }
        .chat-welcome-icon { font-size: 40px; display: block; margin-bottom: 8px; }
        .bubble-row { display: flex; gap: 10px; max-width: 82%; }
        .bubble-row.user { align-self: flex-end; flex-direction: row-reverse; }
        .bubble-avatar { width: 32px; height: 32px; border-radius: 10px; background: var(--bg-surface); display: flex; align-items: center; justify-content: center; font-size: 16px; flex-shrink: 0; border: 1px solid var(--border); }
        .bubble { min-width: 0; }
        .bubble-text {
          padding: 12px 16px; border-radius: var(--radius); line-height: 1.55;
          white-space: pre-wrap; word-break: break-word; font-size: 14px;
        }
        .user .bubble-text { background: linear-gradient(135deg, var(--accent), #5b4fd4); color: white; border-bottom-right-radius: 4px; }
        .assistant .bubble-text { background: var(--bg-surface); border: 1px solid var(--border); border-bottom-left-radius: 4px; }
        .bubble-meta { font-size: 11px; color: var(--text-muted); margin-top: 4px; padding: 0 4px; }
        .typing { display: flex; gap: 4px; padding: 16px 20px; background: var(--bg-surface); border: 1px solid var(--border); border-radius: var(--radius); }
        .typing span { width: 8px; height: 8px; border-radius: 50%; background: var(--accent); animation: bounce 1.2s infinite; }
        .typing span:nth-child(2) { animation-delay: 0.15s; }
        .typing span:nth-child(3) { animation-delay: 0.3s; }
        @keyframes bounce { 0%, 80%, 100% { transform: translateY(0); opacity: 0.4; } 40% { transform: translateY(-6px); opacity: 1; } }
        .chat-att { padding: 6px 24px; display: flex; gap: 6px; flex-wrap: wrap; }
        .chat-composer {
          display: flex; gap: 8px; padding: 16px 24px; border-top: 1px solid var(--border);
          background: var(--bg-elevated); align-items: flex-end;
        }
        .composer-btn {
          width: 40px; height: 40px; border-radius: var(--radius-sm);
          background: var(--bg-surface); border: 1px solid var(--border);
          font-size: 18px; display: flex; align-items: center; justify-content: center;
        }
        .chat-composer textarea { flex: 1; resize: none; min-height: 40px; max-height: 120px; }
        .composer-send { height: 40px; }
      `}</style>
    </div>
  );
}
