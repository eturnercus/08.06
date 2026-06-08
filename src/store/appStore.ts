import { create } from "zustand";
import type { AppSettings } from "../api/tauri";

export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
  tokens?: number;
  latencyMs?: number;
}

export interface ChatPermissions {
  internet: boolean;
  stm: boolean;
  ltm: boolean;
  camera: boolean;
  microphone: boolean;
  screen: boolean;
  files: boolean;
  tools: boolean;
}

export interface Chat {
  id: string;
  title: string;
  modelId: string;
  messages: ChatMessage[];
  permissions: ChatPermissions;
  memoryAccess: string;
  systemPrompt: string;
  agentGroupId?: string;
  ramLimitMb: number;
  maxTokens: number;
  temperature: number;
}

export interface MonitorEvent {
  id: string;
  timestamp: string;
  type: string;
  agentName?: string;
  message: string;
  status: "ok" | "error" | "running";
}

interface AppStore {
  phase: "language" | "onboarding" | "app";
  settings: AppSettings | null;
  chats: Chat[];
  activeChatId: string | null;
  activeView: string;
  onboardingStep: number;
  monitorEvents: MonitorEvent[];
  selectedGroupId: string | null;
  setPhase: (p: "language" | "onboarding" | "app") => void;
  setSettings: (s: AppSettings) => void;
  setActiveView: (v: string) => void;
  setOnboardingStep: (n: number) => void;
  setSelectedGroupId: (id: string | null) => void;
  addChat: () => string;
  setActiveChat: (id: string) => void;
  updateChat: (id: string, patch: Partial<Chat>) => void;
  addMessage: (chatId: string, msg: ChatMessage) => void;
  pushMonitorEvent: (e: MonitorEvent) => void;
  clearMonitor: () => void;
}

const defaultPerms = (): ChatPermissions => ({
  internet: false, stm: true, ltm: true, camera: false,
  microphone: false, screen: false, files: true, tools: true,
});

export const useAppStore = create<AppStore>((set, get) => ({
  phase: "language",
  settings: null,
  chats: [],
  activeChatId: null,
  activeView: "chats",
  onboardingStep: 0,
  monitorEvents: [],
  selectedGroupId: null,
  setPhase: (p) => set({ phase: p }),
  setSettings: (s) => set({ settings: s }),
  setActiveView: (v) => set({ activeView: v }),
  setOnboardingStep: (n) => set({ onboardingStep: n }),
  setSelectedGroupId: (id) => set({ selectedGroupId: id }),
  addChat: () => {
    const id = `chat-${Date.now()}`;
    const n = get().chats.length + 1;
    const chat: Chat = {
      id,
      title: `Chat ${n}`,
      modelId: "default",
      messages: [],
      permissions: defaultPerms(),
      memoryAccess: "CHAT_ONLY",
      systemPrompt: "",
      ramLimitMb: 4096,
      maxTokens: 4096,
      temperature: 0.7,
    };
    set((s) => ({ chats: [...s.chats, chat], activeChatId: id }));
    return id;
  },
  setActiveChat: (id) => set({ activeChatId: id }),
  updateChat: (id, patch) =>
    set((s) => ({
      chats: s.chats.map((c) => (c.id === id ? { ...c, ...patch } : c)),
    })),
  addMessage: (chatId, msg) =>
    set((s) => ({
      chats: s.chats.map((c) =>
        c.id === chatId ? { ...c, messages: [...c.messages, msg] } : c
      ),
    })),
  pushMonitorEvent: (e) =>
    set((s) => ({ monitorEvents: [e, ...s.monitorEvents].slice(0, 200) })),
  clearMonitor: () => set({ monitorEvents: [] }),
}));
