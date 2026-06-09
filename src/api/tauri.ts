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
  systemPromptCustomized?: boolean;
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
      ? invoke<{
          content: string;
          tokensUsed: number;
          promptTokens: number;
          completionTokens: number;
          latencyMs: number;
          memoryRecalled: number;
          injectionApplied: boolean;
          modelId: string;
          maxTokensLimit: number;
        }>("send_chat", { request })
      : Promise.resolve({
          ...browserSendChat(request.message),
          memoryRecalled: 0,
          injectionApplied: false,
          modelId: request.modelId,
        }),
  agentFetch: (url: string, chatId?: string, agentId?: string) =>
    invoke<NetworkLog>("agent_fetch", { url, chatId, agentId }),
  getNetworkLogs: () => invoke<NetworkLog[]>("get_network_logs"),
  webSearch: (query: string, agentId?: string, chatId?: string) =>
    invoke<NetworkLog>("web_search", { query, agentId, chatId }),
  stopChat: (chatId: string) => invoke<void>("stop_chat", { chatId }),
  syncChatOverrides: (p: {
    chatId: string;
    allowInternet: boolean;
    stmEnabled: boolean;
    ltmEnabled: boolean;
    agentGroupId?: string;
    workspacePath?: string;
    ramLimitMb?: number;
    memoryAccess?: string;
  }) => invoke<void>("sync_chat_overrides", p),
  stopAgentTeam: (taskId?: string) => invoke<void>("stop_agent_team", { taskId }),
  stopAgentMember: (taskId: string, agentId: string) =>
    invoke<void>("stop_agent_member", { taskId, agentId }),
  ensureStarterModel: () => invoke<ModelInfo | null>("ensure_starter_model"),
  downloadStarterModel: (force?: boolean) =>
    invoke<DownloadResult>("download_starter_model", { force: force ?? false }),
  searchHuggingfaceModels: (query: string, limit?: number) =>
    invoke<{ id: string; downloads?: number; tags: string[] }[]>("search_huggingface_models", {
      query,
      limit,
    }),
  getLlamaRuntimeStatus: () =>
    invoke<{
      embeddedAvailable: boolean;
      cliPath?: string;
      cliReady: boolean;
      version?: string;
      ggufRuntime: string;
      activeEngine: string;
      message: string;
    }>("get_llama_runtime_status"),
  ensureLlamaRuntime: (force?: boolean) =>
    invoke<{
      embeddedAvailable: boolean;
      cliPath?: string;
      cliReady: boolean;
      version?: string;
      ggufRuntime: string;
      activeEngine: string;
      message: string;
    }>("ensure_llama_runtime", { force: force ?? false }),
  getAuditLogs: (maxLines?: number) => invoke<string[]>("get_audit_logs", { maxLines }),
  openBrowserUrl: (url: string, chatId?: string, agentId?: string) =>
    invoke<NetworkLog>("open_browser_url", { url, chatId, agentId }),
  getDesktopAgentState: () => invoke<DesktopAgentSnapshot>("get_desktop_agent_state"),
  virtualMouseMove: (x: number, y: number, label?: string) =>
    invoke<void>("virtual_mouse_move", { x, y, label }),
  virtualMouseScroll: (deltaY: number) => invoke<void>("virtual_mouse_scroll", { deltaY }),
  browserNavigateInApp: (url: string, chatId?: string, agentId?: string) =>
    invoke<string>("browser_navigate_in_app", { url, chatId, agentId }),
  browserSearchInApp: (query: string, chatId?: string, agentId?: string) =>
    invoke<string>("browser_search_in_app", { query, chatId, agentId }),
  browserClickInApp: (p: {
    linkIndex?: number;
    x?: number;
    y?: number;
    selector?: string;
    chatId?: string;
    agentId?: string;
  }) => invoke<string>("browser_click_in_app", p),
  setAgentWebviewLive: (enabled: boolean) =>
    invoke<AgentWebViewState>("set_agent_webview_live", { enabled }),
  showAgentWebview: () => invoke<void>("show_agent_webview"),
  hideAgentWebview: () => invoke<void>("hide_agent_webview"),
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
  runAgentTeam: (groupId: string, prompt: string, chatId?: string) =>
    invoke<AgentTask>("run_agent_team", { groupId, prompt, chatId }),
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
      : Promise.resolve("~/.local/share/silenium/models"),
  getDeviceStatus: () => invoke<DeviceStatus>("get_device_status"),
  captureScreen: () => invoke<CaptureResult>("capture_screen"),
  captureAudio: () => invoke<CaptureResult>("capture_audio"),
  captureCamera: () => invoke<CaptureResult>("capture_camera"),
  ocrScreen: () => invoke<CaptureResult>("ocr_screen"),
  transcribeAudio: () => invoke<CaptureResult>("transcribe_audio"),
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
  agentId?: string;
  chatId?: string;
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
  finalResponse?: string;
  rounds: { roundNumber: number; messages: { agentId?: string; agentName: string; role: string; content: string; usedInternet: boolean; toolsUsed?: string[] }[] }[];
}

export interface DeviceStatus {
  cameraAvailable: boolean;
  microphoneAvailable: boolean;
  screenCaptureAvailable: boolean;
  virtualDisplayActive: boolean;
  virtualDisplayResolution?: string;
  ocrAvailable?: boolean;
  sttAvailable?: boolean;
}

export interface CaptureResult {
  success: boolean;
  message: string;
  dataBase64?: string;
  mimeType?: string;
  text?: string;
}

export interface AgentWebViewState {
  liveEnabled: boolean;
  windowVisible: boolean;
  url: string;
  title: string;
  lastAction: string;
  domMode: boolean;
}

export interface VirtualMouseState {
  x: number;
  y: number;
  visible: boolean;
  clicking: boolean;
  label: string;
}

export interface BrowserLink {
  index: number;
  text: string;
  href: string;
}

export interface AgentBrowserState {
  url: string;
  title: string;
  htmlSrcdoc: string;
  status: string;
  message: string;
  links: BrowserLink[];
}

export interface DesktopAgentSnapshot {
  dualMouseEnabled: boolean;
  virtualMouse: VirtualMouseState;
  browser: AgentBrowserState;
  webview?: AgentWebViewState;
}
