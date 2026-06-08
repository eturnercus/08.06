import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { isTauri } from "../api/browserFallback";
import { useAppStore } from "../store/appStore";

interface AgentStreamPayload {
  taskId: string;
  agentId: string;
  agentName: string;
  delta: string;
  done: boolean;
}

export function useAgentStream() {
  const appendAgentStreamDelta = useAppStore((s) => s.appendAgentStreamDelta);
  const finalizeAgentStream = useAppStore((s) => s.finalizeAgentStream);

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    let unlistenFn: (() => void) | null = null;
    listen<AgentStreamPayload>("agent-stream", (event) => {
      const p = event.payload;
      if (p.done) {
        finalizeAgentStream(p.taskId, p.agentId);
      } else if (p.delta) {
        appendAgentStreamDelta(p.taskId, p.agentId, p.agentName, p.delta);
      }
    }).then((fn) => {
      if (disposed) fn();
      else unlistenFn = fn;
    });
    return () => {
      disposed = true;
      unlistenFn?.();
    };
  }, [appendAgentStreamDelta, finalizeAgentStream]);
}
