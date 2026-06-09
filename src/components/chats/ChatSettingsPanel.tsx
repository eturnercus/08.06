import { useState } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "../../store/appStore";
import { Tooltip } from "../ui/Tooltip";
import { ModelSelect } from "../models/ModelSelect";
import { MEMORY_ACCESS_LEVELS } from "../../constants/agents";
import { isTauri } from "../../api/browserFallback";
import { useModels } from "../../hooks/useModels";
import { api } from "../../api/tauri";

const STARTER_ID = "silenium-starter";

export function ChatSettingsPanel({ chatId }: { chatId: string }) {
  const { t, i18n } = useTranslation();
  const chat = useAppStore((s) => s.chats.find((c) => c.id === chatId));
  const settings = useAppStore((s) => s.settings);
  const updateChat = useAppStore((s) => s.updateChat);
  const { models, refresh, loading: modelsLoading } = useModels(false);
  const [downloading, setDownloading] = useState(false);
  const [downloadError, setDownloadError] = useState<string | null>(null);
  const lang = i18n.language === "ru" ? "ru" : "en";

  if (!chat) return null;

  const groups = settings?.agentGroups ?? [];
  const teamMode = Boolean(chat.agentGroupId);
  const starter = models.find((m) => m.id === STARTER_ID);
  const starterNeedsDownload = Boolean(
    !teamMode &&
      (chat.modelId === STARTER_ID || chat.modelId === "default") &&
      starter &&
      (!starter.path || !starter.loaded)
  );

  const toggle = (key: keyof typeof chat.permissions) => {
    updateChat(chatId, { permissions: { ...chat.permissions, [key]: !chat.permissions[key] } });
  };

  const setMode = (mode: "single" | "team") => {
    if (mode === "single") {
      updateChat(chatId, {
        agentGroupId: undefined,
        workspacePath: undefined,
        modelId: chat.modelId === "default" ? STARTER_ID : chat.modelId,
      });
    } else if (groups.length > 0) {
      updateChat(chatId, { agentGroupId: chat.agentGroupId ?? groups[0].id });
    }
  };

  const downloadStarter = async () => {
    setDownloading(true);
    setDownloadError(null);
    try {
      const result = await api.downloadStarterModel(true);
      if (!result.success) {
        setDownloadError(result.message);
        return;
      }
      await refresh();
      updateChat(chatId, { modelId: STARTER_ID });
    } catch (e) {
      setDownloadError(String(e));
    } finally {
      setDownloading(false);
    }
  };

  return (
    <div className="chat-settings-panel scroll-y">
      <div className="chat-settings-header">{t("chat.settingsTitle")}</div>

      <div className="form-row">
        <label className="form-label">{t("chat.chatTitle")}</label>
        <input className="m3-input" value={chat.title} onChange={(e) => updateChat(chatId, { title: e.target.value })} />
      </div>

      <div className="form-row">
        <label className="form-label">{t("chat.responseMode")}</label>
        <div className="chat-mode-toggle" role="group" aria-label={t("chat.responseMode")}>
          <button
            type="button"
            className={`chat-mode-btn${!teamMode ? " active" : ""}`}
            onClick={() => setMode("single")}
          >
            {t("chat.modeSingle")}
          </button>
          <button
            type="button"
            className={`chat-mode-btn${teamMode ? " active" : ""}`}
            onClick={() => setMode("team")}
            disabled={groups.length === 0}
            title={groups.length === 0 ? t("chat.modeTeamDisabled") : undefined}
          >
            {t("chat.modeTeam")}
          </button>
        </div>
        <p className="field-hint">
          {teamMode ? t("chat.teamModelHint") : t("chat.singleModelHint")}
        </p>
      </div>

      {!teamMode && (
        <>
          <ModelSelect
            label={t("models.select")}
            value={chat.modelId === "default" ? STARTER_ID : chat.modelId}
            onChange={(modelId) => updateChat(chatId, { modelId })}
          />
          {(chat.modelId === STARTER_ID || chat.modelId === "default") && (
            <p className="field-hint">{t("chat.starterLimitHint")}</p>
          )}
          {starterNeedsDownload && isTauri() && (
            <div className="form-row starter-download-row">
              <p className="field-hint">{t("chat.starterDownloadHint")}</p>
              <button
                type="button"
                className="m3-filled-btn"
                disabled={downloading || modelsLoading}
                onClick={downloadStarter}
              >
                {downloading ? t("models.downloading") : t("chat.starterDownloadBtn")}
              </button>
              {downloadError && <p className="field-error">{downloadError}</p>}
            </div>
          )}
        </>
      )}

      {teamMode && groups.length > 0 && (
        <div className="form-row">
          <label className="form-label">{t("chat.agentGroup")}</label>
          <select
            className="m3-input"
            value={chat.agentGroupId ?? ""}
            onChange={(e) => updateChat(chatId, { agentGroupId: e.target.value || undefined })}
          >
            {groups.map((g) => (
              <option key={g.id} value={g.id}>
                {g.name}
              </option>
            ))}
          </select>
        </div>
      )}

      <div className="form-row">
        <label className="form-label">{t("chat.systemPrompt")}</label>
        <textarea
          className="m3-input scroll-y"
          rows={2}
          value={chat.systemPrompt}
          onChange={(e) => updateChat(chatId, { systemPrompt: e.target.value })}
          placeholder={t("chat.systemPromptPh")}
        />
      </div>

      {!teamMode && (
      <>
      <div className="form-row">
        <label className="form-label">{t("chat.genPreset")}</label>
        <div className="gen-preset-row" role="group">
          {(
            [
              ["creative", 1.0],
              ["balanced", 0.7],
              ["precise", 0.3],
              ["code", 0.15],
            ] as const
          ).map(([key, temp]) => (
            <button
              key={key}
              type="button"
              className={`gen-preset-btn${Math.abs(chat.temperature - temp) < 0.05 ? " active" : ""}`}
              onClick={() => updateChat(chatId, { temperature: temp })}
            >
              {t(`chat.preset${key.charAt(0).toUpperCase()}${key.slice(1)}` as "chat.presetCreative")}
            </button>
          ))}
        </div>
      </div>
      <div className="chat-settings-grid">
        <div>
          <label className="form-label">{t("chat.temperature")}</label>
          <input
            type="number"
            step={0.1}
            min={0}
            max={2}
            className="m3-input"
            value={chat.temperature}
            onChange={(e) => updateChat(chatId, { temperature: Number(e.target.value) })}
          />
        </div>
        <div>
          <label className="form-label" title={t("chat.maxTokensReplyHint")}>
            {t("chat.maxTokensReply")} ⓘ
          </label>
          <input
            type="number"
            min={64}
            max={2048}
            step={64}
            className="m3-input"
            value={chat.maxTokens}
            onChange={(e) =>
              updateChat(chatId, { maxTokens: Math.min(2048, Math.max(64, Number(e.target.value))) })
            }
          />
        </div>
        <div>
          <label className="form-label">{t("chat.ramLimit")}</label>
          <input
            type="number"
            className="m3-input"
            value={chat.ramLimitMb}
            onChange={(e) => updateChat(chatId, { ramLimitMb: Number(e.target.value) })}
          />
        </div>
      </div>
      </>
      )}

      {teamMode && (
        <p className="field-hint team-params-hint">{t("chat.teamParamsHint")}</p>
      )}

      {teamMode && chat.agentGroupId && isTauri() && (
        <div className="form-row workspace-block">
          <label className="form-label">{t("chat.workspaceFolder")}</label>
          <p className="field-hint">{t("chat.workspaceHint")}</p>
          <div className="workspace-picker">
            <input
              className="m3-input mono"
              readOnly
              value={chat.workspacePath ?? ""}
              placeholder={t("chat.workspacePh")}
            />
            <button
              type="button"
              className="m3-tonal-btn"
              onClick={async () => {
                const selected = await open({ directory: true, multiple: false });
                if (typeof selected === "string") {
                  updateChat(chatId, { workspacePath: selected });
                }
              }}
            >
              {t("chat.workspacePick")}
            </button>
            {chat.workspacePath && (
              <button
                type="button"
                className="m3-outlined-btn"
                onClick={() => updateChat(chatId, { workspacePath: undefined })}
              >
                ✕
              </button>
            )}
          </div>
        </div>
      )}

      <div className="form-row">
        <label className="form-label">{t("memory.accessLevel")}</label>
        <select
          className="m3-input"
          value={chat.memoryAccess}
          onChange={(e) => updateChat(chatId, { memoryAccess: e.target.value })}
        >
          {MEMORY_ACCESS_LEVELS.map((l) => (
            <option key={l.id} value={l.id}>
              {l[lang]}
            </option>
          ))}
        </select>
      </div>

      <div className="form-row">
        <label className="form-label">{t("chat.permsTitle")}</label>
        <div className="perm-chips">
          {chat.permissions.internet && (
            <p className="field-hint">{t("chat.internetSearchHint")}</p>
          )}
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
      </div>
    </div>
  );
}
