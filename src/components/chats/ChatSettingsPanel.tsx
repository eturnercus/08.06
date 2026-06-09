import { useTranslation } from "react-i18next";
import { useAppStore } from "../../store/appStore";
import { Tooltip } from "../ui/Tooltip";
import { ModelSelect } from "../models/ModelSelect";
import { MEMORY_ACCESS_LEVELS } from "../../constants/agents";

export function ChatSettingsPanel({ chatId }: { chatId: string }) {
  const { t, i18n } = useTranslation();
  const chat = useAppStore((s) => s.chats.find((c) => c.id === chatId));
  const settings = useAppStore((s) => s.settings);
  const updateChat = useAppStore((s) => s.updateChat);
  const lang = i18n.language === "ru" ? "ru" : "en";

  if (!chat) return null;

  const toggle = (key: keyof typeof chat.permissions) => {
    updateChat(chatId, { permissions: { ...chat.permissions, [key]: !chat.permissions[key] } });
  };

  const groups = settings?.agentGroups ?? [];

  return (
    <div className="chat-settings-panel scroll-y">
      <div className="chat-settings-header">{t("chat.settingsTitle")}</div>

      <div className="form-row">
        <label className="form-label">{t("chat.chatTitle")}</label>
        <input className="m3-input" value={chat.title} onChange={(e) => updateChat(chatId, { title: e.target.value })} />
      </div>

      <ModelSelect
        label={t("models.select")}
        value={chat.modelId}
        onChange={(modelId) => updateChat(chatId, { modelId })}
      />

      <div className="form-row">
        <label className="form-label">{t("chat.systemPrompt")}</label>
        <textarea className="m3-input scroll-y" rows={2} value={chat.systemPrompt}
          onChange={(e) => updateChat(chatId, { systemPrompt: e.target.value })}
          placeholder={t("chat.systemPromptPh")} />
      </div>

      <div className="chat-settings-grid">
        <div>
          <label className="form-label">{t("chat.temperature")}</label>
          <input type="number" step={0.1} min={0} max={2} className="m3-input" value={chat.temperature}
            onChange={(e) => updateChat(chatId, { temperature: Number(e.target.value) })} />
        </div>
        <div>
          <label className="form-label">{t("chat.maxTokens")}</label>
          <input type="number" className="m3-input" value={chat.maxTokens}
            onChange={(e) => updateChat(chatId, { maxTokens: Number(e.target.value) })} />
        </div>
        <div>
          <label className="form-label">{t("chat.ramLimit")}</label>
          <input type="number" className="m3-input" value={chat.ramLimitMb}
            onChange={(e) => updateChat(chatId, { ramLimitMb: Number(e.target.value) })} />
        </div>
      </div>

      {groups.length > 0 && (
        <div className="form-row">
          <label className="form-label">{t("chat.agentGroup")}</label>
          <select className="m3-input" value={chat.agentGroupId ?? ""}
            onChange={(e) => updateChat(chatId, { agentGroupId: e.target.value || undefined })}>
            <option value="">{t("chat.noAgentGroup")}</option>
            {groups.map((g) => <option key={g.id} value={g.id}>{g.name}</option>)}
          </select>
        </div>
      )}

      <div className="form-row">
        <label className="form-label">{t("memory.accessLevel")}</label>
        <select className="m3-input" value={chat.memoryAccess}
          onChange={(e) => updateChat(chatId, { memoryAccess: e.target.value })}>
          {MEMORY_ACCESS_LEVELS.map((l) => <option key={l.id} value={l.id}>{l[lang]}</option>)}
        </select>
      </div>

      <div className="form-row">
        <label className="form-label">{t("chat.permsTitle")}</label>
        <div className="perm-chips">
          {(["internet", "stm", "ltm", "camera", "microphone", "screen", "files", "tools"] as const).map((k) => (
            <Tooltip key={k} text={t(`chat.permTip.${k}`)}>
              <button type="button" className={`m3-chip ${chat.permissions[k] ? "active" : ""}`} onClick={() => toggle(k)}>
                {t(`chat.perm.${k}`)}
              </button>
            </Tooltip>
          ))}
        </div>
      </div>
    </div>
  );
}
