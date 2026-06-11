import { invoke } from "@tauri-apps/api/core";
import {
  browserGetSettings,
  browserListModels,
  browserSendChat,
  browserUpdateSettings,
  isTauri,
} from "./browserFallback";

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

export interface ModelInfo {
  id: string;
  name: string;
  path?: string;
  format: string;
  source: string;
  loaded: boolean;
  sizeBytes?: number;
  verified?: boolean;
  downloadProgress?: number;
}

export interface DownloadResult {
  success: boolean;
  model?: ModelInfo;
  message: string;
  bytesDownloaded: number;
  verified: boolean;
}

export const api = {
  getSettings: () =>
    isTauri() ? invoke<AppSettings>("get_settings") : Promise.resolve(browserGetSettings()),
  updateSettings: (s: AppSettings) =>
    isTauri()
      ? invoke<void>("update_settings", { settings: s })
      : (browserUpdateSettings(s), Promise.resolve()),
  resetSettings: () => invoke<AppSettings>("reset_settings_cmd"),
  sendChat: (request: {
    chatId: string;
    modelId: string;
    message: string;
    systemPrompt?: string;
    temperature?: number;
    maxTokens?: number;
    attachments: { name: string; mimeType: string; sizeBytes: number; dataBase64?: string }[];
  }) =>
    isTauri()
      ? invoke<{ content: string; tokensUsed: number; latencyMs: number; memoryRecalled: number; injectionApplied: boolean; modelId: string }>("send_chat", { request })
      : Promise.resolve({ ...browserSendChat(request.message), memoryRecalled: 0, injectionApplied: false, modelId: request.modelId }),
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
    invoke<LtmEntry | null>("consolidate_memory", { chatId, modelId }),
  bridgeMemoryModels: (p: { chatId: string; fromModel: string; toModel: string }) =>
    invoke<MemoryBridgeResult | null>("bridge_memory_models", p),
  getMemoryOverview: (chatId: string) =>
    invoke<MemoryOverview>("get_memory_overview", { chatId }),
  runAgentTeam: (groupId: string, prompt: string) =>
    invoke<AgentTask>("run_agent_team", { groupId, prompt }),
  listAgentTasks: () => invoke<AgentTask[]>("list_agent_tasks"),
  loadModel: (path: string, name: string) => invoke<ModelInfo>("load_model", { path, name }),
  downloadHuggingfaceModel: (repo: string) => invoke<DownloadResult>("download_huggingface_model", { repo }),
  scanLocalModels: () => invoke<ModelInfo[]>("scan_local_models"),
  verifyModel: (modelId: string) => invoke<boolean>("verify_model", { modelId }),
  listModels: () =>
    isTauri() ? invoke<ModelInfo[]>("list_models") : Promise.resolve(browserListModels()),
  getModelsDirectory: () =>
    isTauri()
      ? invoke<string>("get_models_directory")
      : Promise.resolve("~/.local/share/neuroforge/models"),
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

export interface MemoryBridgeResult {
  bridgeId: string;
  stmMessagesBridged: number;
  summaryChars: number;
  fromModelId: string;
  toModelId: string;
}

export interface MemoryOverview {
  stmCount: number;
  ltmCount: number;
  bridgeCount: number;
  crossModelReady: boolean;
}

export interface AgentTask {
  id: string;
  status: string;
  prompt: string;
  orchestrationMode?: string;
  rounds: { roundNumber: number; messages: { agentName: string; role: string; content: string; usedInternet: boolean; toolsUsed?: string[] }[] }[];
}

export interface DeviceStatus {
  cameraAvailable: boolean;
  microphoneAvailable: boolean;
  screenCaptureAvailable: boolean;
  virtualDisplayActive: boolean;
}
