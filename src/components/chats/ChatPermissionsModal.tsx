import { useTranslation } from "react-i18next";
import { useAppStore } from "../../store/appStore";
import { Tooltip } from "../ui/Tooltip";
import { MEMORY_ACCESS_LEVELS } from "../../constants/agents";

export function ChatPermissionsModal({ chatId }: { chatId: string }) {
  const { t, i18n } = useTranslation();
  const chat = useAppStore((s) => s.chats.find((c) => c.id === chatId));
  const updateChat = useAppStore((s) => s.updateChat);
  const lang = i18n.language === "ru" ? "ru" : "en";

  if (!chat) return null;

  const toggle = (key: keyof typeof chat.permissions) => {
    updateChat(chatId, { permissions: { ...chat.permissions, [key]: !chat.permissions[key] } });
  };

  return (
    <div style={{ padding: 12, borderTop: "1px solid var(--m3-outline-variant)" }}>
      <div style={{ fontSize: 12, fontWeight: 600, marginBottom: 8 }}>{t("chat.permsTitle")}</div>
      <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
        {(["internet", "stm", "ltm", "camera", "microphone", "screen", "files", "tools"] as const).map((k) => (
          <Tooltip key={k} text={t(`chat.permTip.${k}`)}>
            <button
              type="button"
              className={`m3-chip ${chat.permissions[k] ? "active" : ""}`}
              onClick={() => toggle(k)}
            >
              {t(`chat.perm.${k}`)}
            </button>
          </Tooltip>
        ))}
      </div>
      <div className="form-row" style={{ marginTop: 10 }}>
        <label className="form-label">{t("memory.accessLevel")}</label>
        <select
          className="m3-input"
          value={chat.memoryAccess}
          onChange={(e) => updateChat(chatId, { memoryAccess: e.target.value })}
        >
          {MEMORY_ACCESS_LEVELS.map((l) => (
            <option key={l.id} value={l.id}>{l[lang]}</option>
          ))}
        </select>
      </div>
    </div>
  );
}
