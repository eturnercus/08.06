import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import { useModels } from "../hooks/useModels";
import { useChatStream } from "../hooks/useChatStream";
import { useChatScroll } from "../hooks/useChatScroll";
import { MessageBubble } from "./chat/MessageBubble";
import { ChatSettingsPanel } from "./chats/ChatSettingsPanel";
import { api } from "../api/tauri";
import { isTauri } from "../api/browserFallback";
import { MediaCapture, MediaAttachment } from "./chat/MediaCapture";
import { EmptyState } from "./ui/EmptyState";

export function ChatView() {
  const { t } = useTranslation();
  const {
    chats,
    activeChatId,
    addChat,
    addMessage,
    finalizeStreamMessage,
    setActiveGeneration,
    settings,
  } = useAppStore();
  useChatStream();
  const { models } = useModels();
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);
  const [attachments, setAttachments] = useState<MediaAttachment[]>([]);

  const chat = chats.find((c) => c.id === activeChatId);
  const modelName = models.find((m) => m.id === chat?.modelId)?.name ?? chat?.modelId;
  const agentGroup = (settings?.agentGroups as { id: string; name: string }[] | undefined)?.find(
    (g) => g.id === chat?.agentGroupId
  );

  useEffect(() => {
    setInput("");
    setAttachments([]);
  }, [activeChatId]);

  const { containerRef, onScroll, onFocus } = useChatScroll([
    chat?.id,
    chat?.messages.length,
    chat?.messages[chat.messages.length - 1]?.content,
    chat?.messages[chat.messages.length - 1]?.content?.length,
  ]);

  const streamOn =
    isTauri() &&
    Boolean(
      (settings as { inference?: { streaming?: boolean }; innovation?: { thoughtStreaming?: boolean } } | null)
        ?.inference?.streaming ||
        (settings as { innovation?: { thoughtStreaming?: boolean } } | null)?.innovation?.thoughtStreaming
    );

  const syncOverrides = async () => {
    if (!chat || !isTauri()) return;
    await api.syncChatOverrides({
      chatId: chat.id,
      allowInternet: chat.permissions.internet,
      stmEnabled: chat.permissions.stm,
      ltmEnabled: chat.permissions.ltm,
      agentGroupId: chat.agentGroupId,
      workspacePath: chat.workspacePath,
      ramLimitMb: chat.ramLimitMb,
      memoryAccess: chat.memoryAccess,
    });
  };

  const handleStop = async () => {
    if (!chat || !isTauri()) return;
    if (chat.agentGroupId) {
      await api.stopAgentTeam().catch(() => {});
    }
    await api.stopChat(chat.id);
    finalizeStreamMessage(chat.id, { cancelled: true });
    setLoading(false);
    setActiveGeneration(null);
  };

  const handleSend = async () => {
    if ((!input.trim() && attachments.length === 0) || !chat) return;
    const userText = input.trim() || t("chat.mediaOnly");
    const sentAttachments = [...attachments];
    setInput("");
    setAttachments([]);
    setLoading(true);
    setActiveGeneration(chat.id);
    addMessage(chat.id, {
      role: "user",
      content: userText,
      attachments: sentAttachments.map((a) => ({
        name: a.name,
        mimeType: a.mimeType,
        sizeBytes: a.sizeBytes,
      })),
    });
    await syncOverrides();

    const useAgentTeam = Boolean(chat.agentGroupId && isTauri());
    if (streamOn && !useAgentTeam) {
      addMessage(chat.id, { role: "assistant", content: "", streaming: true });
    }

    try {
      if (useAgentTeam) {
        const task = await api.runAgentTeam(chat.agentGroupId!, userText, chat.id);
        const finalText =
          task.finalResponse?.trim() ||
          task.rounds.at(-1)?.messages.at(-1)?.content ||
          t("chat.agentNoResponse");
        addMessage(chat.id, {
          role: "assistant",
          agentName: agentGroup?.name ?? t("chat.agentTeam"),
          content: finalText,
          meta: {
            team: true,
            rounds: task.rounds.length,
            status: task.status,
            agents: task.rounds.reduce((n, r) => n + r.messages.length, 0),
          },
        });
      } else {
        const resp = await api.sendChat({
          chatId: chat.id,
          modelId: chat.modelId,
          message: userText,
          systemPrompt: chat.systemPrompt || undefined,
          temperature: chat.temperature,
          maxTokens: chat.maxTokens,
          attachments: sentAttachments.map((a) => ({
            name: a.name,
            mimeType: a.mimeType,
            sizeBytes: a.sizeBytes,
            dataBase64: a.dataBase64,
          })),
        });
        if (!streamOn) {
          addMessage(chat.id, {
            role: "assistant",
            content: resp.content,
            tokens: resp.completionTokens ?? resp.tokensUsed,
            promptTokens: resp.promptTokens,
            completionTokens: resp.completionTokens ?? resp.tokensUsed,
            latencyMs: resp.latencyMs,
            meta: {
              modelId: resp.modelId,
              promptTokens: resp.promptTokens,
              completionTokens: resp.completionTokens ?? resp.tokensUsed,
              maxTokensLimit: resp.maxTokensLimit,
              memoryRecalled: resp.memoryRecalled,
              injection: resp.injectionApplied,
            },
          });
        }
      }
    } catch (e) {
      const err = String(e);
      const stopped = err.includes("остановлена") || err.includes("cancelled");
      if (streamOn || stopped) {
        finalizeStreamMessage(chat.id, {
          cancelled: stopped,
          error: stopped ? undefined : `Error: ${e}`,
        });
      } else {
        addMessage(chat.id, { role: "assistant", content: t("chat.errorGeneric", { err: String(e) }) });
      }
    }
    setLoading(false);
    setActiveGeneration(null);
  };

  if (!chat) {
    return (
      <EmptyState
        icon="💬"
        title={t("chat.emptyTitle")}
        description={t("chat.emptyDesc")}
        action={
          <button type="button" className="m3-filled-btn" onClick={() => addChat()}>
            + {t("chat.newChat")}
          </button>
        }
      />
    );
  }

  return (
    <div className="chat" onFocus={onFocus} tabIndex={-1}>
      <div className="chat-toolbar">
        <div className="chat-title-wrap">
          <h2>{chat.title}</h2>
          <div className="chat-badges">
            {agentGroup ? (
              <span className="badge badge-purple">🧩 {agentGroup.name}</span>
            ) : (
              <span className="badge badge-purple">🧠 {modelName}</span>
            )}
            {chat.permissions.stm && <span className="badge badge-blue">{t("chat.stm")}</span>}
            {chat.permissions.ltm && <span className="badge badge-cyan">{t("chat.ltm")}</span>}
            {chat.permissions.internet
              ? <span className="badge badge-green">{t("chat.internet")}</span>
              : <span className="badge badge-red">{t("chat.offline")}</span>}
          </div>
        </div>
        <div className="chat-toolbar-actions">
          <button
            type="button"
            className={`m3-outlined-btn chat-settings-toggle${settingsOpen ? " active" : ""}`}
            onClick={() => setSettingsOpen((v) => !v)}
            aria-expanded={settingsOpen}
          >
            ⚙ {t("chat.settingsTitle")}
          </button>
          <button type="button" className="m3-outlined-btn" onClick={() => addChat()}>+ {t("chat.newChat")}</button>
        </div>
      </div>

      <div
        ref={containerRef}
        className="chat-messages scroll-y"
        onScroll={onScroll}
        onClick={onFocus}
      >
        {chat.messages.length === 0 && (
          <div className="chat-welcome">
            <span className="chat-welcome-icon" aria-hidden>👋</span>
            <h3 className="chat-welcome-title">{t("chat.welcomeTitle")}</h3>
            <ul className="chat-welcome-steps">
              <li>{t("chat.welcomeStep1")}</li>
              <li>{t("chat.welcomeStep2")}</li>
              <li>{t("chat.welcomeStep3")}</li>
            </ul>
          </div>
        )}
        {chat.messages.map((m) => (
          <MessageBubble key={m.id} message={m} />
        ))}
        {loading && !chat.messages.some((m) => m.streaming) && (
          <div className="bubble-row assistant">
            <div className="bubble-avatar">🤖</div>
            <div className="bubble">
              <div className="bubble-text" style={{ opacity: 0.85 }}>{t("chat.generating")}</div>
              <div className="typing" style={{ border: "none", padding: "8px 0 0" }}><span /><span /><span /></div>
            </div>
          </div>
        )}
      </div>

      <div className={`chat-settings-drawer${settingsOpen ? "" : " collapsed"}`}>
        <ChatSettingsPanel chatId={chat.id} />
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
            next.push({
              name: f.name,
              mimeType: f.type || "application/octet-stream",
              sizeBytes: f.size,
              dataBase64: btoa(binary),
              previewUrl: f.type.startsWith("image/") ? URL.createObjectURL(f) : undefined,
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
          {loading ? (
            <button type="button" className="m3-filled-btn composer-send stop-btn" onClick={handleStop}>
              {t("chat.stop")}
            </button>
          ) : (
            <button
              type="button"
              className="m3-filled-btn composer-send"
              onClick={handleSend}
              disabled={!input.trim() && attachments.length === 0}
            >
              {t("chat.send")}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
