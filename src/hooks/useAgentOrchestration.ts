import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useAppStore } from "../store/appStore";
import { isTauri } from "../api/browserFallback";

export interface AgentOrchestrationEvent {
  taskId: string;
  groupId: string;
  groupName: string;
  orchestrationMode: string;
  round: number;
  phase: string;
  agentId?: string;
  agentName?: string;
  modelId?: string;
  status: string;
  message?: string;
}

export function useAgentOrchestration() {
  const pushMonitorEvent = useAppStore((s) => s.pushMonitorEvent);
  const setActiveAgentTask = useAppStore((s) => s.setActiveAgentTask);

  useEffect(() => {
    if (!isTauri()) return;
    const unlisten = listen<AgentOrchestrationEvent>("agent-orchestration", (e) => {
      const p = e.payload;
      if (p.phase === "task_start") {
        setActiveAgentTask({ taskId: p.taskId, groupId: p.groupId, groupName: p.groupName });
      }
      if (p.phase === "task_done") {
        setActiveAgentTask(null);
      }
      pushMonitorEvent({
        id: `orch-${p.taskId}-${p.phase}-${p.agentId ?? "team"}-${Date.now()}`,
        timestamp: new Date().toISOString(),
        type: p.phase,
        taskId: p.taskId,
        agentId: p.agentId,
        agentName: p.agentName,
        message: p.message ?? `${p.groupName} · r${p.round} · ${p.orchestrationMode}${p.modelId ? ` · ${p.modelId}` : ""}`,
        status: p.status === "running" ? "running" : p.status === "error" ? "error" : "ok",
        round: p.round,
        orchestrationMode: p.orchestrationMode,
        modelId: p.modelId,
      });
    });
    return () => {
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, [pushMonitorEvent, setActiveAgentTask]);
}
