import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { isTauri } from "../api/browserFallback";
import { useAppStore } from "../store/appStore";

interface ChatStreamPayload {
  chatId: string;
  delta: string;
  done: boolean;
  tokensUsed?: number;
  promptTokens?: number;
  completionTokens?: number;
  latencyMs?: number;
  modelId?: string;
  memoryRecalled?: number;
  injectionApplied?: boolean;
  maxTokensLimit?: number;
  cancelled?: boolean;
  error?: string;
}

export function useChatStream() {
  const appendStreamDelta = useAppStore((s) => s.appendStreamDelta);
  const finalizeStreamMessage = useAppStore((s) => s.finalizeStreamMessage);

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    let unlistenFn: (() => void) | null = null;
    listen<ChatStreamPayload>("chat-stream", (event) => {
      const p = event.payload;
      if (p.done) {
        finalizeStreamMessage(p.chatId, {
          tokens: p.completionTokens ?? p.tokensUsed,
          promptTokens: p.promptTokens,
          completionTokens: p.completionTokens ?? p.tokensUsed,
          latencyMs: p.latencyMs,
          cancelled: p.cancelled ?? false,
          error: p.cancelled ? undefined : p.error,
          meta: {
            modelId: p.modelId,
            promptTokens: p.promptTokens,
            completionTokens: p.completionTokens ?? p.tokensUsed,
            maxTokensLimit: p.maxTokensLimit,
            memoryRecalled: p.memoryRecalled,
            injection: p.injectionApplied,
          },
        });
      } else if (p.delta) {
        appendStreamDelta(p.chatId, p.delta);
      }
    }).then((fn) => {
      if (disposed) fn();
      else unlistenFn = fn;
    });
    return () => {
      disposed = true;
      unlistenFn?.();
    };
  }, [appendStreamDelta, finalizeStreamMessage]);
}
