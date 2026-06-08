import type { AgentGroup, AgentMember } from "../api/tauri";

export function defaultMember(partial: Partial<AgentMember> & Pick<AgentMember, "id" | "name" | "role">): AgentMember {
  return {
    modelId: "default",
    permissions: {
      internet: false,
      camera: false,
      microphone: false,
      screen: false,
      stm: true,
      ltm: true,
      canDelegate: false,
      files: false,
      tools: true,
      veto: false,
      sharedMemory: true,
    },
    resources: {
      ramLimitMb: 2048,
      cpuCores: [0, 1],
      maxTokens: 2048,
      temperature: 0.7,
      executionOrder: 0,
    },
    tools: ["memory_query", "calculator"],
    trigger: "always",
    triggerKeyword: "",
    systemPrompt: "",
    ...partial,
  };
}

export function createDefaultAgentGroup(name = "Research Team"): AgentGroup {
  return {
    id: "default-team",
    name,
    enabled: true,
    orchestrationMode: "hierarchical",
    members: [
      defaultMember({
        id: "leader",
        name: "Leader",
        role: "leader",
        permissions: {
          internet: false, camera: false, microphone: false, screen: false,
          stm: true, ltm: true, canDelegate: true, files: true, tools: true,
          veto: true, sharedMemory: true,
        },
        tools: ["delegate", "memory_query", "summarize"],
        systemPrompt: "You coordinate the team and resolve conflicts.",
        resources: { ramLimitMb: 4096, cpuCores: [0, 1, 2], maxTokens: 4096, temperature: 0.5, executionOrder: 0 },
      }),
      defaultMember({
        id: "researcher",
        name: "Researcher",
        role: "researcher",
        permissions: {
          internet: true, camera: false, microphone: false, screen: false,
          stm: true, ltm: true, canDelegate: false, files: true, tools: true,
          veto: false, sharedMemory: true,
        },
        tools: ["web_search", "memory_query", "file_read"],
        systemPrompt: "You research topics using web search and files.",
        resources: { ramLimitMb: 3072, cpuCores: [2, 3], maxTokens: 3072, temperature: 0.6, executionOrder: 1 },
      }),
      defaultMember({
        id: "programmer",
        name: "Programmer",
        role: "programmer",
        permissions: {
          internet: false, camera: false, microphone: false, screen: false,
          stm: true, ltm: true, canDelegate: false, files: true, tools: true,
          veto: false, sharedMemory: true,
        },
        tools: ["code_exec", "file_read", "file_write", "json_parse", "regex"],
        systemPrompt: "You write and review code.",
        resources: { ramLimitMb: 4096, cpuCores: [4, 5], maxTokens: 4096, temperature: 0.3, executionOrder: 2 },
      }),
    ],
    sharedMemory: true,
    maxRounds: 5,
    parallelExecution: false,
    consensusThreshold: 0.75,
    conflictMode: "consensus",
    timeoutSec: 120,
    feedbackLoops: true,
    taskDecomposition: true,
  };
}
