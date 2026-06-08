import { create } from "zustand";
import type { AppSettings } from "../api/tauri";

export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
  tokens?: number;
  latencyMs?: number;
}

export interface Chat {
  id: string;
  title: string;
  modelId: string;
  messages: ChatMessage[];
  permissions: {
    internet: boolean;
    stm: boolean;
    ltm: boolean;
    camera: boolean;
    microphone: boolean;
    screen: boolean;
  };
}

interface AppStore {
  phase: "language" | "onboarding" | "app";
  settings: AppSettings | null;
  chats: Chat[];
  activeChatId: string | null;
  activeView: string;
  onboardingStep: number;
  setPhase: (p: "language" | "onboarding" | "app") => void;
  setSettings: (s: AppSettings) => void;
  setActiveView: (v: string) => void;
  setOnboardingStep: (n: number) => void;
  addChat: () => string;
  setActiveChat: (id: string) => void;
  addMessage: (chatId: string, msg: ChatMessage) => void;
}

export const useAppStore = create<AppStore>((set, get) => ({
  phase: "language",
  settings: null,
  chats: [],
  activeChatId: null,
  activeView: "chats",
  onboardingStep: 0,
  setPhase: (p) => set({ phase: p }),
  setSettings: (s) => set({ settings: s }),
  setActiveView: (v) => set({ activeView: v }),
  setOnboardingStep: (n) => set({ onboardingStep: n }),
  addChat: () => {
    const id = `chat-${Date.now()}`;
    const chat: Chat = {
      id,
      title: `Chat ${get().chats.length + 1}`,
      modelId: "default",
      messages: [],
      permissions: {
        internet: false,
        stm: true,
        ltm: true,
        camera: false,
        microphone: false,
        screen: false,
      },
    };
    set((s) => ({ chats: [...s.chats, chat], activeChatId: id }));
    return id;
  },
  setActiveChat: (id) => set({ activeChatId: id }),
  addMessage: (chatId, msg) =>
    set((s) => ({
      chats: s.chats.map((c) =>
        c.id === chatId ? { ...c, messages: [...c.messages, msg] } : c
      ),
    })),
}));
