import { invoke } from "@tauri-apps/api/core";

export interface AppSettings {
  language: string;
  firstRunCompleted: boolean;
  onboardingStep: number;
  system: Record<string, unknown>;
  network: Record<string, unknown>;
  memory: Record<string, unknown>;
  inference: Record<string, unknown>;
  devices: Record<string, unknown>;
  ui: Record<string, unknown>;
  advanced: Record<string, unknown>;
  globalMessageInjection: Record<string, unknown>;
  perChatOverrides: Record<string, unknown>;
  agentGroups: AgentGroup[];
  customModels: unknown[];
}

export interface AgentGroup {
  id: string;
  name: string;
  enabled: boolean;
  orchestrationMode: string;
  members: AgentMember[];
  sharedMemory: boolean;
  maxRounds: number;
  parallelExecution: boolean;
  consensusThreshold?: number;
  conflictMode?: string;
  timeoutSec?: number;
  feedbackLoops?: boolean;
  taskDecomposition?: boolean;
}

export interface AgentMember {
  id: string;
  name: string;
  role: string;
  modelId: string;
  permissions: {
    internet: boolean;
    camera: boolean;
    microphone: boolean;
    screen: boolean;
    stm: boolean;
    ltm: boolean;
    canDelegate: boolean;
    files?: boolean;
    tools?: boolean;
    veto?: boolean;
    sharedMemory?: boolean;
  };
  resources?: {
    ramLimitMb: number;
    cpuCores: number[];
    maxTokens: number;
    temperature: number;
    executionOrder: number;
  };
  tools?: string[];
  trigger?: string;
  triggerKeyword?: string;
  systemPrompt?: string;
}

export const api = {
  getSettings: () => invoke<AppSettings>("get_settings"),
  updateSettings: (s: AppSettings) => invoke<void>("update_settings", { settings: s }),
  resetSettings: () => invoke<AppSettings>("reset_settings_cmd"),
  sendChat: (request: {
    chatId: string;
    modelId: string;
    message: string;
    attachments: { name: string; mimeType: string; sizeBytes: number; dataBase64?: string }[];
  }) => invoke<{ content: string; tokensUsed: number; latencyMs: number; memoryRecalled: number; injectionApplied: boolean }>("send_chat", { request }),
  agentFetch: (url: string, chatId?: string, agentId?: string) =>
    invoke<NetworkLog>("agent_fetch", { url, chatId, agentId }),
  getNetworkLogs: () => invoke<NetworkLog[]>("get_network_logs"),
  webSearch: (query: string, agentId?: string) => invoke<NetworkLog>("web_search", { query, agentId }),
  getMemoryStm: (chatId: string) => invoke<StmEntry[]>("get_memory_stm", { chatId }),
  getMemoryLtm: (chatId?: string) => invoke<LtmEntry[]>("get_memory_ltm", { chatId }),
  transferMemory: (p: {
    entryIds: string[];
    fromChat: string;
    toChat: string;
    fromModel: string;
    toModel: string;
    memoryType: string;
  }) => invoke("transfer_memory", p),
  consolidateMemory: (chatId: string, modelId: string) =>
    invoke("consolidate_memory", { chatId, modelId }),
  runAgentTeam: (groupId: string, prompt: string) =>
    invoke<AgentTask>("run_agent_team", { groupId, prompt }),
  listAgentTasks: () => invoke<AgentTask[]>("list_agent_tasks"),
  loadModel: (path: string, name: string) => invoke("load_model", { path, name }),
  loadHuggingfaceModel: (repo: string) => invoke("load_huggingface_model", { repo }),
  listModels: () => invoke<ModelInfo[]>("list_models"),
  getDeviceStatus: () => invoke<DeviceStatus>("get_device_status"),
  captureScreen: () => invoke("capture_screen"),
  captureAudio: () => invoke("capture_audio"),
  captureCamera: () => invoke("capture_camera"),
  getSystemInfo: () => invoke<Record<string, unknown>>("get_system_info"),
};

export interface NetworkLog {
  id: string;
  url: string;
  method: string;
  status?: number;
  blocked: boolean;
  blockReason?: string;
  responsePreview: string;
  durationMs: number;
  timestamp: string;
}

export interface StmEntry {
  role: string;
  content: string;
  tokens: number;
  timestamp: string;
}

export interface LtmEntry {
  id: string;
  content: string;
  memoryType: string;
  importance: number;
  transferable: boolean;
  chatId: string;
}

export interface AgentTask {
  id: string;
  status: string;
  prompt: string;
  orchestrationMode?: string;
  rounds: { roundNumber: number; messages: { agentName: string; role: string; content: string; usedInternet: boolean; toolsUsed?: string[] }[] }[];
}

export interface ModelInfo {
  id: string;
  name: string;
  format: string;
  source: string;
  loaded: boolean;
}

export interface DeviceStatus {
  cameraAvailable: boolean;
  microphoneAvailable: boolean;
  screenCaptureAvailable: boolean;
  virtualDisplayActive: boolean;
}
