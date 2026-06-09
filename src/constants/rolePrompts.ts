import type { AgentMember } from "../api/tauri";

export const ROLE_DEFAULT_PROMPTS: Record<string, { ru: string; en: string }> = {
  leader: {
    ru: "Ты лидер команды агентов. Координируй задачу, распределяй подзадачи, собирай выводы в один согласованный результат. Пиши кратко и по делу.",
    en: "You are the team leader. Coordinate the task, delegate subtasks, and merge outputs into one coherent result. Be concise.",
  },
  worker: {
    ru: "Ты исполнитель. Выполняй конкретную подзадачу из контекста команды. Отвечай структурированно, без лишней воды.",
    en: "You are a worker agent. Execute the assigned subtask from team context. Reply in a structured, concise way.",
  },
  researcher: {
    ru: "Ты исследователь. Ищи факты, сравнивай источники, отмечай неопределённость. Используй веб-поиск при необходимости.",
    en: "You are a researcher. Find facts, compare sources, note uncertainty. Use web search when needed.",
  },
  analyst: {
    ru: "Ты аналитик. Разбей задачу на шаги, выдели риски, метрики и выводы. Таблицы и списки приветствуются.",
    en: "You are an analyst. Break the task into steps, risks, metrics, and conclusions. Use lists and tables.",
  },
  programmer: {
    ru: "Ты программист. Пиши рабочий код, объясняй решения, учитывай ограничения среды. Файлы — только в рабочей папке чата.",
    en: "You are a programmer. Write working code, explain decisions, respect environment limits. Files only in chat workspace.",
  },
  reviewer: {
    ru: "Ты рецензент. Проверяй логику, качество, безопасность и полноту ответов других агентов. Указывай конкретные правки.",
    en: "You are a reviewer. Check logic, quality, safety, and completeness. Give concrete fixes.",
  },
  summarizer: {
    ru: "Ты составитель итогового резюме. Объедини все ответы команды в один финальный ответ для пользователя без дублирования.",
    en: "You synthesize the team output into one final user-facing answer without duplication.",
  },
  translator: {
    ru: "Ты переводчик. Сохраняй смысл, тон и термины. Указывай язык источника и результата.",
    en: "You are a translator. Preserve meaning, tone, and terms. State source and target languages.",
  },
  creative: {
    ru: "Ты креативный агент. Генерируй идеи, сценарии и формулировки. Отделяй факты от гипотез.",
    en: "You are a creative agent. Generate ideas and wording. Separate facts from hypotheses.",
  },
  fact_checker: {
    ru: "Ты проверяющий фактов. Верифицируй утверждения, помечай неподтверждённое, предлагай источники.",
    en: "You fact-check claims, mark unverified statements, and suggest sources.",
  },
  router: {
    ru: "Ты маршрутизатор. Определи, какой тип эксперта нужен для следующего шага, и сформулируй узкую подзадачу.",
    en: "You route work: decide which expert is needed next and phrase a narrow subtask.",
  },
  custom_manager: {
    ru: "Ты пользовательский менеджер. Следуй целям пользователя, управляй приоритетами и сроками команды.",
    en: "You are a custom manager. Follow user goals and manage team priorities.",
  },
};

export const ROLE_DEFAULT_TOKENS: Record<string, number> = {
  leader: 3072,
  summarizer: 4096,
  programmer: 4096,
  researcher: 3072,
  analyst: 2560,
  reviewer: 2048,
  creative: 3072,
  translator: 2048,
  fact_checker: 2048,
  router: 1024,
  worker: 2048,
  custom_manager: 3072,
};

export function roleDefaultPrompt(roleId: string, lang: "ru" | "en"): string {
  return ROLE_DEFAULT_PROMPTS[roleId]?.[lang] ?? ROLE_DEFAULT_PROMPTS.worker[lang];
}

const LEGACY_EN_PROMPTS: Record<string, string> = {
  leader: "You coordinate the team and resolve conflicts.",
  researcher: "You research topics using web search and files.",
  programmer: "You write and review code.",
};

export function isAutoPromptMember(
  member: AgentMember & { systemPromptCustomized?: boolean },
  lang: "ru" | "en"
): boolean {
  if (member.systemPromptCustomized) return false;
  const p = (member.systemPrompt || "").trim();
  if (!p) return true;
  const ru = roleDefaultPrompt(member.role, "ru");
  const en = roleDefaultPrompt(member.role, "en");
  if (p === ru || p === en) return true;
  if (LEGACY_EN_PROMPTS[member.role] === p) return true;
  return false;
}

export function applyRoleDefaults(
  member: AgentMember & { systemPromptCustomized?: boolean },
  roleId: string,
  lang: "ru" | "en"
): Partial<AgentMember & { systemPromptCustomized?: boolean }> {
  const patch: Partial<AgentMember & { systemPromptCustomized?: boolean }> = { role: roleId };
  if (isAutoPromptMember(member, lang)) {
    patch.systemPrompt = roleDefaultPrompt(roleId, lang);
    patch.systemPromptCustomized = false;
  }
  const suggested = ROLE_DEFAULT_TOKENS[roleId];
  if (suggested && member.resources) {
    patch.resources = { ...member.resources, maxTokens: suggested };
  }
  return patch;
}
