import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { ChatMessage } from "../../store/appStore";

export function MessageBubble({ message }: { message: ChatMessage }) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const isAssistant = message.role === "assistant";
  const liveTokens =
    message.streamTokens ??
    (message.streaming ? Math.max(1, Math.floor(message.content.length / 4)) : message.tokens);

  return (
    <div className={`bubble-row ${message.role}`}>
      <div className="bubble-avatar">
        {message.role === "user" ? "👤" : message.agentName ? "🧩" : "🤖"}
      </div>
      <div className="bubble">
        {isAssistant && (
          <button
            type="button"
            className="bubble-meta-toggle"
            onClick={() => setOpen((v) => !v)}
            aria-expanded={open}
          >
            <span className="mono">
              {message.streaming ? "▶ " : "▼ "}
              {liveTokens != null && `${liveTokens} tok`}
              {message.latencyMs != null && ` · ${message.latencyMs}ms`}
              {message.cancelled && ` · ${t("chat.stopped")}`}
            </span>
            {message.agentName && (
              <span className="badge badge-purple" style={{ marginLeft: 8 }}>
                {message.agentName}
              </span>
            )}
          </button>
        )}
        {open && isAssistant && message.meta && (
          <div className="bubble-meta-panel mono">
            {Object.entries(message.meta).map(([k, v]) =>
              v != null && v !== "" ? (
                <div key={k}>
                  <strong>{k}:</strong> {String(v)}
                </div>
              ) : null
            )}
          </div>
        )}
        {message.thinking && (
          <details className="thinking-block" open={message.streaming}>
            <summary>{t("chat.thinking")}</summary>
            <pre className="thinking-text">{message.thinking}</pre>
          </details>
        )}
        <div className="bubble-text">
          {message.content}
          {message.streaming && <span className="stream-cursor">▍</span>}
        </div>
        {!open && message.tokens !== undefined && !message.streaming && (
          <div className="bubble-meta mono">
            {t("chat.tokens")}: {message.tokens}
            {message.latencyMs != null && ` · ${t("chat.latency")}: ${message.latencyMs}ms`}
          </div>
        )}
      </div>
    </div>
  );
}
