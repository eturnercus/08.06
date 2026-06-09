import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { ChatMessage } from "../../store/appStore";

const META_LABELS: Record<string, string> = {
  modelId: "chat.meta.model",
  promptTokens: "chat.meta.promptTokens",
  completionTokens: "chat.meta.completionTokens",
  maxTokensLimit: "chat.meta.maxTokensLimit",
  memoryRecalled: "chat.meta.memoryRecalled",
  injection: "chat.meta.injection",
  team: "chat.meta.team",
  rounds: "chat.meta.rounds",
  status: "chat.meta.status",
  agents: "chat.meta.agents",
  stopped: "chat.meta.stopped",
};

export function MessageBubble({ message }: { message: ChatMessage }) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const isAssistant = message.role === "assistant";

  const completion =
    message.completionTokens ?? message.tokens ?? (message.streaming ? undefined : undefined);
  const prompt = message.promptTokens;

  const hasProps =
    isAssistant &&
    (completion != null ||
      prompt != null ||
      message.latencyMs != null ||
      message.cancelled ||
      (message.meta && Object.keys(message.meta).length > 0));

  const metaLabel = (key: string) => {
    const path = META_LABELS[key];
    return path ? t(path) : key;
  };

  return (
    <div className={`bubble-row ${message.role}`}>
      <div className="bubble-avatar">
        {message.role === "user" ? "👤" : message.agentName ? "🧩" : "🤖"}
      </div>
      <div className="bubble">
        {hasProps && (
          <button
            type="button"
            className="bubble-meta-toggle"
            onClick={() => setOpen((v) => !v)}
            aria-expanded={open}
          >
            <span className="mono">
              {open ? "▼" : "▶"} {t("chat.messageProps")}
              {message.streaming && ` · ${t("chat.generatingShort")}`}
              {!message.streaming && completion != null && ` · ${completion} ${t("chat.tokensOut")}`}
              {message.latencyMs != null && !message.streaming && ` · ${message.latencyMs}ms`}
              {message.cancelled && ` · ${t("chat.stopped")}`}
            </span>
            {message.agentName && (
              <span className="badge badge-purple" style={{ marginLeft: 8 }}>
                {message.agentName}
              </span>
            )}
          </button>
        )}
        {open && hasProps && (
          <div className="bubble-meta-panel">
            {prompt != null && (
              <div>
                <strong>{t("chat.meta.promptTokens")}:</strong> {prompt}
              </div>
            )}
            {completion != null && (
              <div>
                <strong>{t("chat.meta.completionTokens")}:</strong> {completion}
              </div>
            )}
            {message.latencyMs != null && (
              <div>
                <strong>{t("chat.latency")}:</strong> {message.latencyMs} ms
              </div>
            )}
            {message.cancelled && (
              <div>
                <strong>{t("chat.meta.stopped")}:</strong> {t("chat.yes")}
              </div>
            )}
            {message.meta &&
              Object.entries(message.meta).map(([k, v]) =>
                v != null && v !== "" && k !== "promptTokens" && k !== "completionTokens" ? (
                  <div key={k}>
                    <strong>{metaLabel(k)}:</strong> {String(v)}
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
      </div>
    </div>
  );
}
