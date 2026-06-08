import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import { useModels } from "../hooks/useModels";
import { api } from "../api/tauri";
import { MediaCapture, MediaAttachment } from "./chat/MediaCapture";

export function ChatView() {
  const { t } = useTranslation();
  const { chats, activeChatId, addChat, addMessage } = useAppStore();
  const { models } = useModels();
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);
  const [attachments, setAttachments] = useState<MediaAttachment[]>([]);

  const chat = chats.find((c) => c.id === activeChatId);
  const modelName = models.find((m) => m.id === chat?.modelId)?.name ?? chat?.modelId;

  const handleSend = async () => {
    if ((!input.trim() && attachments.length === 0) || !chat) return;
    setLoading(true);
    const userText = input.trim() || t("chat.mediaOnly");
    addMessage(chat.id, { role: "user", content: userText });
    try {
      const resp = await api.sendChat({
        chatId: chat.id,
        modelId: chat.modelId,
        message: userText,
        systemPrompt: chat.systemPrompt || undefined,
        temperature: chat.temperature,
        maxTokens: chat.maxTokens,
        attachments: attachments.map((a) => ({
          name: a.name,
          mimeType: a.mimeType,
          sizeBytes: a.sizeBytes,
          dataBase64: a.dataBase64,
        })),
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

  if (!chat) {
    return (
      <div className="chat-empty">
        <div className="chat-empty-icon">💬</div>
        <h2>{t("chat.newChat")}</h2>
        <button type="button" className="m3-filled-btn" onClick={() => addChat()}>{t("chat.newChat")}</button>
      </div>
    );
  }

  return (
    <div className="chat">
      <div className="chat-toolbar">
        <div className="chat-title-wrap">
          <h2>{chat.title}</h2>
          <div className="chat-badges">
            <span className="badge badge-purple">🧠 {modelName}</span>
            {chat.permissions.stm && <span className="badge badge-blue">{t("chat.stm")}</span>}
            {chat.permissions.ltm && <span className="badge badge-cyan">{t("chat.ltm")}</span>}
            {chat.permissions.internet
              ? <span className="badge badge-green">{t("chat.internet")}</span>
              : <span className="badge badge-red">{t("chat.offline")}</span>}
            {chat.permissions.camera && <span className="badge badge-green">📷</span>}
            {chat.permissions.microphone && <span className="badge badge-green">🎤</span>}
          </div>
        </div>
        <button type="button" className="m3-outlined-btn" onClick={() => addChat()}>+ {t("chat.newChat")}</button>
      </div>

      <div className="chat-messages scroll-y">
        {chat.messages.length === 0 && (
          <div className="chat-welcome">
            <span className="chat-welcome-icon">🧠</span>
            <p>{t("chat.welcomeHint")}</p>
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
          {attachments.map((a, i) => (
            <span key={i} className="badge badge-blue">
              {a.previewUrl ? <img src={a.previewUrl} alt="" className="att-thumb" /> : null}
              {a.name}
            </span>
          ))}
        </div>
      )}

      <div className="chat-composer">
        <input ref={fileRef} type="file" multiple hidden onChange={async (e) => {
          const files = e.target.files;
          if (!files) return;
          const next: MediaAttachment[] = [];
          for (const f of Array.from(files)) {
            const buf = await f.arrayBuffer();
            const bytes = new Uint8Array(buf);
            let binary = "";
            bytes.forEach((b) => { binary += String.fromCharCode(b); });
            const dataBase64 = btoa(binary);
            const previewUrl = f.type.startsWith("image/") ? URL.createObjectURL(f) : undefined;
            next.push({
              name: f.name,
              mimeType: f.type || "application/octet-stream",
              sizeBytes: f.size,
              dataBase64,
              previewUrl,
            });
          }
          setAttachments((prev) => [...prev, ...next]);
          e.target.value = "";
        }} accept="image/*,audio/*,video/*,.pdf" />
        <div className="chat-composer-row">
          <div className="composer-tools">
            <button type="button" className="composer-media-btn" onClick={() => fileRef.current?.click()} title={t("chat.attach")}>📎</button>
            <MediaCapture
              cameraEnabled={chat.permissions.camera}
              micEnabled={chat.permissions.microphone}
              onAttach={(a) => setAttachments((p) => [...p, a])}
            />
          </div>
          <textarea
            className="m3-input composer-input"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder={t("chat.placeholder")}
            rows={2}
            onKeyDown={(e) => { if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); handleSend(); } }}
          />
          <button type="button" className="m3-filled-btn composer-send" onClick={handleSend} disabled={loading || (!input.trim() && attachments.length === 0)}>
            {t("chat.send")}
          </button>
        </div>
      </div>
    </div>
  );
}
