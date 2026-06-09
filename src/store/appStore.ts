import { create } from "zustand";
import type { AppSettings } from "../api/tauri";
import { sanitizeLlmOutput } from "../utils/sanitizeLlm";

export interface MessageAttachment {
  name: string;
  mimeType: string;
  sizeBytes: number;
}

export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  attachments?: MessageAttachment[];
  tokens?: number;
  promptTokens?: number;
  completionTokens?: number;
  latencyMs?: number;
  streaming?: boolean;
  streamTokens?: number;
  thinking?: string;
  cancelled?: boolean;
  agentName?: string;
  meta?: Record<string, string | number | boolean | undefined>;
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
  workspacePath?: string;
  ramLimitMb: number;
  maxTokens: number;
  temperature: number;
}

export interface MonitorEvent {
  id: string;
  timestamp: string;
  type: string;
  agentName?: string;
  agentId?: string;
  taskId?: string;
  message: string;
  status: "ok" | "error" | "running";
  streaming?: boolean;
  round?: number;
  orchestrationMode?: string;
  modelId?: string;
}

export interface ActiveAgentTask {
  taskId: string;
  groupId: string;
  groupName: string;
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
  activeGenerationChatId: string | null;
  activeAgentTask: ActiveAgentTask | null;
  setPhase: (p: "language" | "onboarding" | "app") => void;
  setSettings: (s: AppSettings) => void;
  setActiveView: (v: string) => void;
  setOnboardingStep: (n: number) => void;
  setSelectedGroupId: (id: string | null) => void;
  addChat: () => string;
  setActiveChat: (id: string) => void;
  updateChat: (id: string, patch: Partial<Chat>) => void;
  addMessage: (chatId: string, msg: Omit<ChatMessage, "id"> & { id?: string }) => void;
  deleteChat: (chatId: string) => void;
  exportChat: (chatId: string) => string;
  setActiveGeneration: (chatId: string | null) => void;
  appendStreamDelta: (chatId: string, delta: string) => void;
  finalizeStreamMessage: (
    chatId: string,
    patch: {
      content?: string;
      tokens?: number;
      promptTokens?: number;
      completionTokens?: number;
      latencyMs?: number;
      error?: string;
      cancelled?: boolean;
      meta?: Record<string, string | number | boolean | undefined>;
    }
  ) => void;
  /** Close stuck streaming placeholders before a new send. */
  closeOrphanStreamMessages: (chatId: string) => void;
  pushMonitorEvent: (e: MonitorEvent) => void;
  appendAgentStreamDelta: (
    taskId: string,
    agentId: string,
    agentName: string,
    delta: string
  ) => void;
  finalizeAgentStream: (taskId: string, agentId: string) => void;
  clearMonitor: () => void;
  setActiveAgentTask: (task: ActiveAgentTask | null) => void;
  loadChats: () => void;
  persistChats: () => void;
}

const CHATS_KEY = "silenium-chats";
const ACTIVE_KEY = "silenium-active-chat";

const defaultPerms = (): ChatPermissions => ({
  internet: false, stm: true, ltm: true, camera: true,
  microphone: true, screen: false, files: true, tools: true,
});

function splitThinkingStream(combined: string): { thinking?: string; content: string } {
  const openTag = "<" + "think" + ">";
  const closeTag = "<" + "/" + "think" + ">";
  const lower = combined.toLowerCase();
  const open = lower.indexOf(openTag);
  const close = lower.indexOf(closeTag);
  if (open >= 0 && close > open) {
    return {
      thinking: combined.slice(open + openTag.length, close).trim(),
      content: combined.slice(close + closeTag.length).trim(),
    };
  }
  if (open >= 0 && close < 0) {
    return { thinking: combined.slice(open + openTag.length).trim(), content: "" };
  }
  return { content: combined };
}

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
  activeGenerationChatId: null,
  activeAgentTask: null,
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
        const chats = (JSON.parse(raw) as Chat[]).map((c) => ({
          ...c,
          modelId: c.modelId === "default" ? "silenium-starter" : c.modelId,
          maxTokens: c.maxTokens >= 4096 ? 512 : c.maxTokens,
          messages: c.messages.map((m, i) => ({
            ...m,
            id: m.id ?? `legacy-${c.id}-${i}`,
            content:
              m.role === "assistant" && m.content
                ? sanitizeLlmOutput(m.content)
                : m.content,
          })),
        }));
        const activeChatId =
          active && chats.some((c) => c.id === active) ? active : chats[0]?.id ?? null;
        set({ chats, activeChatId });
        persist(chats, activeChatId);
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
      modelId: "silenium-starter",
      messages: [],
      permissions: defaultPerms(),
      memoryAccess: "CHAT_ONLY",
      systemPrompt: "",
      ramLimitMb: 4096,
      maxTokens: 512,
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
  deleteChat: (chatId) =>
    set((s) => {
      const chats = s.chats.filter((c) => c.id !== chatId);
      const activeChatId =
        s.activeChatId === chatId ? chats[0]?.id ?? null : s.activeChatId;
      persist(chats, activeChatId);
      return { chats, activeChatId };
    }),
  exportChat: (chatId) => {
    const chat = get().chats.find((c) => c.id === chatId);
    if (!chat) return "";
    return JSON.stringify(chat, null, 2);
  },
  setActiveGeneration: (chatId) => set({ activeGenerationChatId: chatId }),
  addMessage: (chatId, msg) =>
    set((s) => {
      const full: ChatMessage = {
        ...msg,
        id: msg.id ?? `msg-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
        content:
          msg.role === "assistant" && msg.content
            ? sanitizeLlmOutput(msg.content)
            : msg.content,
      };
      const chats = s.chats.map((c) =>
        c.id === chatId ? { ...c, messages: [...c.messages, full] } : c
      );
      persist(chats, s.activeChatId);
      return { chats };
    }),
  appendStreamDelta: (chatId, delta) =>
    set((s) => {
      const chats = s.chats.map((c) => {
        if (c.id !== chatId) return c;
        const messages = [...c.messages];
        for (let i = messages.length - 1; i >= 0; i--) {
          if (messages[i].role === "assistant" && messages[i].streaming) {
            const combined = messages[i].content + delta;
            const split = splitThinkingStream(combined);
            messages[i] = {
              ...messages[i],
              content: sanitizeLlmOutput(split.content),
              thinking: split.thinking ?? messages[i].thinking,
            };
            break;
          }
        }
        return { ...c, messages };
      });
      persist(chats, s.activeChatId);
      return { chats };
    }),
  closeOrphanStreamMessages: (chatId) =>
    set((s) => {
      const chats = s.chats.map((c) => {
        if (c.id !== chatId) return c;
        let changed = false;
        const messages = c.messages.map((m) => {
          if (m.role === "assistant" && m.streaming) {
            changed = true;
            return {
              ...m,
              streaming: false,
              cancelled: !m.content?.trim(),
            };
          }
          return m;
        });
        return changed ? { ...c, messages } : c;
      });
      persist(chats, s.activeChatId);
      return { chats };
    }),
  finalizeStreamMessage: (chatId, patch) =>
    set((s) => {
      const chats = s.chats.map((c) => {
        if (c.id !== chatId) return c;
        const messages = [...c.messages];
        let updated = false;
        const applyPatch = (idx: number) => {
          const prev = messages[idx];
          const hasBody = Boolean(prev.content?.trim() || prev.thinking?.trim());
          const rawContent =
            patch.content ?? (hasBody ? prev.content : (patch.error ?? prev.content));
          messages[idx] = {
            ...prev,
            streaming: false,
            streamTokens: undefined,
            content: sanitizeLlmOutput(rawContent),
            thinking: prev.thinking,
            tokens: patch.completionTokens ?? patch.tokens ?? prev.tokens,
            promptTokens: patch.promptTokens ?? prev.promptTokens,
            completionTokens: patch.completionTokens ?? patch.tokens ?? prev.completionTokens,
            latencyMs: patch.latencyMs ?? prev.latencyMs,
            cancelled: patch.cancelled ?? prev.cancelled,
            meta: {
              ...prev.meta,
              ...patch.meta,
              ...(patch.cancelled ? { stopped: true } : {}),
            },
          };
          updated = true;
        };
        for (let i = messages.length - 1; i >= 0; i--) {
          if (messages[i].role === "assistant" && messages[i].streaming) {
            applyPatch(i);
            break;
          }
        }
        if (!updated && patch.content?.trim()) {
          for (let i = messages.length - 1; i >= 0; i--) {
            if (messages[i].role === "assistant" && !messages[i].content?.trim()) {
              applyPatch(i);
              break;
            }
          }
        }
        return { ...c, messages };
      });
      persist(chats, s.activeChatId);
      return { chats };
    }),
  pushMonitorEvent: (e) =>
    set((s) => ({ monitorEvents: [e, ...s.monitorEvents].slice(0, 200) })),
  appendAgentStreamDelta: (taskId, agentId, agentName, delta) =>
    set((s) => {
      const idx = s.monitorEvents.findIndex(
        (e) => e.taskId === taskId && e.agentId === agentId && e.streaming
      );
      if (idx >= 0) {
        const events = [...s.monitorEvents];
        events[idx] = {
          ...events[idx],
          message: sanitizeLlmOutput(events[idx].message + delta),
        };
        return { monitorEvents: events };
      }
      return {
        monitorEvents: [
          {
            id: `stream-${taskId}-${agentId}`,
            timestamp: new Date().toISOString(),
            type: "agent",
            agentName,
            agentId,
            taskId,
            message: delta,
            status: "running" as const,
            streaming: true,
          },
          ...s.monitorEvents,
        ].slice(0, 200),
      };
    }),
  finalizeAgentStream: (taskId, agentId) =>
    set((s) => ({
      monitorEvents: s.monitorEvents.map((e) =>
        e.taskId === taskId && e.agentId === agentId && e.streaming
          ? { ...e, streaming: false, status: "ok" as const }
          : e
      ),
    })),
  clearMonitor: () => set({ monitorEvents: [] }),
  setActiveAgentTask: (task) => set({ activeAgentTask: task }),
}));
