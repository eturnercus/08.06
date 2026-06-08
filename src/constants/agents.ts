export const ORCHESTRATION_STRATEGIES = [
  { id: "sequential", ru: "Последовательная", en: "Sequential" },
  { id: "round_robin", ru: "Круговая", en: "Round Robin" },
  { id: "parallel", ru: "Параллельная", en: "Parallel" },
  { id: "hierarchical", ru: "Иерархическая", en: "Hierarchical" },
  { id: "voting", ru: "Голосование", en: "Voting" },
  { id: "debate", ru: "Дебаты", en: "Debate" },
  { id: "chain_of_thought", ru: "Цепочка мыслей", en: "Chain of Thought" },
  { id: "smart_router", ru: "Умный маршрутизатор", en: "Smart Router" },
  { id: "map_reduce", ru: "MapReduce", en: "MapReduce" },
  { id: "expert_panel", ru: "Экспертная панель", en: "Expert Panel" },
] as const;

export const AGENT_ROLES = [
  { id: "leader", ru: "Лидер", en: "Leader" },
  { id: "worker", ru: "Работник", en: "Worker" },
  { id: "researcher", ru: "Исследователь", en: "Researcher" },
  { id: "analyst", ru: "Аналитик", en: "Analyst" },
  { id: "programmer", ru: "Программист", en: "Programmer" },
  { id: "reviewer", ru: "Рецензент", en: "Reviewer" },
  { id: "summarizer", ru: "Составитель резюме", en: "Summarizer" },
  { id: "translator", ru: "Переводчик", en: "Translator" },
  { id: "creative", ru: "Творческий", en: "Creative" },
  { id: "fact_checker", ru: "Проверяющий факты", en: "Fact Checker" },
  { id: "router", ru: "Маршрутизатор", en: "Router" },
  { id: "custom_manager", ru: "Пользовательский менеджер", en: "Custom Manager" },
] as const;

export const AGENT_TOOLS = [
  { id: "web_search", ru: "Веб-поиск", en: "Web Search" },
  { id: "file_read", ru: "Чтение файлов", en: "File Read" },
  { id: "file_write", ru: "Запись файлов", en: "File Write" },
  { id: "code_exec", ru: "Выполнение кода", en: "Code Execution" },
  { id: "summarize", ru: "Суммирование", en: "Summarize" },
  { id: "translate", ru: "Перевод", en: "Translate" },
  { id: "image_analyze", ru: "Анализ изображений", en: "Image Analysis" },
  { id: "audio_transcribe", ru: "Расшифровка аудио", en: "Audio Transcription" },
  { id: "memory_query", ru: "Запрос памяти", en: "Memory Query" },
  { id: "memory_save", ru: "Сохранение в память", en: "Memory Save" },
  { id: "delegate", ru: "Делегирование", en: "Delegate" },
  { id: "calculator", ru: "Калькулятор", en: "Calculator" },
  { id: "json_parse", ru: "Разбор JSON", en: "JSON Parse" },
  { id: "regex", ru: "Регулярные выражения", en: "Regex" },
] as const;

export const TRIGGER_CONDITIONS = [
  { id: "always", ru: "Всегда", en: "Always" },
  { id: "keyword", ru: "Ключевое слово", en: "Keyword" },
  { id: "delegation", ru: "Делегирование", en: "Delegation" },
  { id: "round", ru: "Раунд", en: "Round" },
  { id: "failure", ru: "Сбой", en: "Failure" },
  { id: "custom", ru: "Пользовательский", en: "Custom" },
] as const;

export const CONFLICT_MODES = [
  { id: "leader_decides", ru: "Решает лидер", en: "Leader decides" },
  { id: "voting", ru: "Голосование", en: "Voting" },
  { id: "consensus", ru: "Консенсус", en: "Consensus" },
  { id: "merge", ru: "Слияние", en: "Merge" },
  { id: "retry", ru: "Повтор", en: "Retry" },
  { id: "escalate_user", ru: "Эскалация пользователю", en: "Escalate to user" },
] as const;

export const MEMORY_ACCESS_LEVELS = [
  { id: "CHAT_ONLY", ru: "Только чат", en: "Chat only" },
  { id: "MODEL_SHARED", ru: "Общая для модели", en: "Model shared" },
  { id: "GLOBAL", ru: "Глобальная", en: "Global" },
] as const;
