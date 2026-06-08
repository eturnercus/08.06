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
  loadChats: () => void;
  persistChats: () => void;
}

const CHATS_KEY = "neuroforge-chats";
const ACTIVE_KEY = "neuroforge-active-chat";

const defaultPerms = (): ChatPermissions => ({
  internet: false, stm: true, ltm: true, camera: true,
  microphone: true, screen: false, files: true, tools: true,
});

const persist = (chats: Chat[], activeChatId: string | null) => {
  try {
    localStorage.setItem(CHATS_KEY, JSON.stringify(chats));
    if (activeChatId) localStorage.setItem(ACTIVE_KEY, activeChatId);
  } catch { /* quota */ }
};

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
  loadChats: () => {
    try {
      const raw = localStorage.getItem(CHATS_KEY);
      const active = localStorage.getItem(ACTIVE_KEY);
      if (raw) {
        const chats = JSON.parse(raw) as Chat[];
        set({ chats, activeChatId: active && chats.some((c) => c.id === active) ? active : chats[0]?.id ?? null });
      }
    } catch { /* ignore */ }
  },
  persistChats: () => {
    const { chats, activeChatId } = get();
    persist(chats, activeChatId);
  },
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
    set((s) => {
      const chats = [...s.chats, chat];
      persist(chats, id);
      return { chats, activeChatId: id };
    });
    return id;
  },
  setActiveChat: (id) => {
    set({ activeChatId: id });
    persist(get().chats, id);
  },
  updateChat: (id, patch) =>
    set((s) => {
      const chats = s.chats.map((c) => (c.id === id ? { ...c, ...patch } : c));
      persist(chats, s.activeChatId);
      return { chats };
    }),
  addMessage: (chatId, msg) =>
    set((s) => {
      const chats = s.chats.map((c) =>
        c.id === chatId ? { ...c, messages: [...c.messages, msg] } : c
      );
      persist(chats, s.activeChatId);
      return { chats };
    }),
  pushMonitorEvent: (e) =>
    set((s) => ({ monitorEvents: [e, ...s.monitorEvents].slice(0, 200) })),
  clearMonitor: () => set({ monitorEvents: [] }),
}));
