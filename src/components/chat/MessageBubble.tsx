import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { ChatMessage } from "../../store/appStore";
import { sanitizeLlmOutput } from "../../utils/sanitizeLlm";

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
  const isUser = message.role === "user";

  const completion =
    message.completionTokens ?? message.tokens ?? undefined;
  const prompt = message.promptTokens;
  const hasAttachments = Boolean(message.attachments?.length);

  const displayContent = sanitizeLlmOutput(message.content || "");

  const hasProps =
    (isAssistant &&
      (completion != null ||
        prompt != null ||
        message.latencyMs != null ||
        message.cancelled ||
        (message.meta && Object.keys(message.meta).length > 0))) ||
    (isUser && hasAttachments);

  const metaLabel = (key: string) => {
    const path = META_LABELS[key];
    return path ? t(path) : key;
  };

  return (
    <div className={`bubble-row ${message.role}`}>
      <div className="bubble-avatar">
        {isUser ? "👤" : message.agentName ? "🧩" : "🤖"}
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
              {!message.streaming && isAssistant && completion != null &&
                ` · ${completion} ${t("chat.tokensOut")}`}
              {message.latencyMs != null && !message.streaming && isAssistant &&
                ` · ${message.latencyMs}ms`}
              {message.cancelled && ` · ${t("chat.stopped")}`}
              {isUser && hasAttachments &&
                ` · ${message.attachments!.length} ${t("chat.attachmentsCount")}`}
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
            {isUser && hasAttachments && (
              <div className="attachment-list">
                <strong>{t("chat.attachments")}:</strong>
                <ul>
                  {message.attachments!.map((a, i) => (
                    <li key={i}>
                      {a.name}{" "}
                      <span className="mono">
                        ({a.mimeType}, {(a.sizeBytes / 1024).toFixed(1)} KB)
                      </span>
                    </li>
                  ))}
                </ul>
              </div>
            )}
            {isAssistant && prompt != null && (
              <div>
                <strong>{t("chat.meta.promptTokens")}:</strong> {prompt}
              </div>
            )}
            {isAssistant && completion != null && (
              <div>
                <strong>{t("chat.meta.completionTokens")}:</strong> {completion}
              </div>
            )}
            {isAssistant && message.latencyMs != null && (
              <div>
                <strong>{t("chat.latency")}:</strong> {message.latencyMs} ms
              </div>
            )}
            {message.cancelled && (
              <div>
                <strong>{t("chat.meta.stopped")}:</strong> {t("chat.yes")}
              </div>
            )}
            {isAssistant &&
              message.meta &&
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
          {displayContent}
          {message.streaming && <span className="stream-cursor">▍</span>}
        </div>
      </div>
    </div>
  );
}
