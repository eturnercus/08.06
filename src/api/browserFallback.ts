import type { AppSettings, ModelInfo } from "./tauri";

const SETTINGS_KEY = "neuroforge-browser-settings";

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function baseSettings(): AppSettings {
  return {
    language: "",
    firstRunCompleted: false,
    onboardingStep: 0,
    system: { ramLimitMb: 8192 },
    network: {},
    memory: {},
    inference: {},
    devices: {},
    ui: {},
    advanced: {},
    globalMessageInjection: {},
    perChatOverrides: {},
    agentGroups: [],
    customModels: [],
  };
}

export function browserGetSettings(): AppSettings {
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (raw) return { ...baseSettings(), ...JSON.parse(raw) };
  } catch { /* ignore */ }
  return baseSettings();
}

export function browserUpdateSettings(s: AppSettings): void {
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(s));
}

export function browserListModels(): ModelInfo[] {
  return [{
    id: "default",
    name: "Default (browser preview)",
    format: "gguf",
    source: "builtin",
    loaded: true,
    verified: true,
  }];
}

export function browserSendChat(message: string): { content: string; tokensUsed: number; latencyMs: number } {
  return {
    content: `Превью без Tauri: для реального ответа модели запустите собранное приложение. Вы написали: «${message.slice(0, 200)}»`,
    tokensUsed: Math.max(1, Math.round(message.length / 4)),
    latencyMs: 12,
  };
}
