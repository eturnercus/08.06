import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import { api, AgentGroup, AppSettings } from "../api/tauri";
import i18n from "../i18n";
import { applyUiTheme, THEME_OPTIONS } from "../utils/theme";
import {
  SectionTitle, SettingNumber, SettingSelect, SettingSlider,
  SettingText, SettingToggle,
} from "./ui/SettingField";
import {
  InnovationPanel, InjectionPanel, MemoryPanel,
  NetworkPanel, PerformancePanel, SecurityPanel, SystemPanel,
} from "./settings/SettingsPanels";
import "./ui/ui.css";

/* 9 категорий по ТЗ */
const TABS = [
  { id: "ram", icon: "💾" },
  { id: "cpu", icon: "⚡" },
  { id: "inference", icon: "🧠" },
  { id: "injection", icon: "✨" },
  { id: "memory", icon: "🧬" },
  { id: "internet", icon: "🌐" },
  { id: "permissions", icon: "🔐" },
  { id: "appearance", icon: "🎨" },
  { id: "advanced", icon: "🔧" },
] as const;

export function SettingsView() {
  const { t } = useTranslation();
  const { settings, setSettings } = useAppStore();
  const [tab, setTab] = useState("ram");
  const [draft, setDraft] = useState(settings);
  const [saved, setSaved] = useState(false);
  const [search, setSearch] = useState("");
  const uiDraft = draft?.ui ?? {};

  useEffect(() => {
    if (!draft) return;
    applyUiTheme(uiDraft);
    return () => {
      applyUiTheme(useAppStore.getState().settings?.ui ?? {});
    };
  }, [draft, uiDraft.theme, uiDraft.fontSize, uiDraft.compactMode]);

  if (!draft) return null;

  const d = draft as unknown as Record<string, Record<string, unknown>>;

  const update = (section: string, key: string, value: unknown) => {
    setDraft((prev) => {
      if (!prev) return prev;
      const rec = prev as unknown as Record<string, Record<string, unknown>>;
      const sec = rec[section] ?? {};
      return { ...prev, [section]: { ...sec, [key]: value } } as typeof prev;
    });
    setSaved(false);
  };

  const updateUiLive = async (key: string, value: unknown) => {
    if (!draft) return;
    const nextUi = { ...(draft.ui ?? {}), [key]: value };
    const next = { ...draft, ui: nextUi } as AppSettings;
    setDraft(next);
    applyUiTheme(nextUi);
    await api.updateSettings(next);
    setSettings(next);
    setSaved(true);
  };

  const save = async () => {
    await api.updateSettings(draft as never);
    setSettings(draft);
    if (draft.language !== i18n.language) {
      i18n.changeLanguage(draft.language);
      localStorage.setItem("silenium-lang", draft.language);
    }
    setSaved(true);
  };

  const reset = async () => {
    if (!confirm(t("settings.resetConfirm"))) return;
    const defaults = await api.resetSettings();
    setDraft(defaults);
    setSettings(defaults);
    i18n.changeLanguage(defaults.language);
  };

  const filteredTabs = useMemo(() => {
    if (!search.trim()) return [...TABS];
    const q = search.toLowerCase();
    return TABS.filter((tb) => t(`settings.tabs.${tb.id}`).toLowerCase().includes(q));
  }, [search, t]);

  const inf = d.inference ?? {};
  const dev = d.devices ?? {};
  const adv = d.advanced ?? {};
  const ui = d.ui ?? {};

  return (
    <div className="settings">
      <div className="settings-toolbar">
        <input className="settings-search" placeholder={t("settings.search")} value={search} onChange={(e) => setSearch(e.target.value)} />
        <div className="settings-toolbar-actions">
          {saved && <span className="badge badge-green">✓ {t("settings.saved")}</span>}
          <button className="btn btn-danger" onClick={reset}>{t("settings.reset")}</button>
          <button className="btn btn-primary" onClick={save}>{t("settings.save")}</button>
        </div>
      </div>
      <div className="settings-body">
        <nav className="settings-nav scroll-y">
          {filteredTabs.map((tb) => (
            <button key={tb.id} className={`settings-nav-item ${tab === tb.id ? "active" : ""} ${"highlight" in tb && tb.highlight ? "highlight" : ""}`} onClick={() => setTab(tb.id)}>
              <span>{tb.icon}</span>{t(`settings.tabs.${tb.id}`)}
            </button>
          ))}
        </nav>
        <div className="settings-content scroll-y">
          {tab === "ram" && (
            <>
              <SystemPanel d={d} u={update} />
              <PerformancePanel d={d} u={update} />
            </>
          )}
          {tab === "cpu" && <SystemPanel d={d} u={update} />}
          {tab === "memory" && (
            <>
              <MemoryPanel d={d} u={update} />
              <InnovationPanel d={d} u={update} />
            </>
          )}
          {tab === "internet" && (
            <>
              <NetworkPanel d={d} u={update} />
              <SecurityPanel d={d} u={update} />
            </>
          )}
          {tab === "permissions" && (
            <>
              <SecurityPanel d={d} u={update} />
              <SectionTitle>{t("settings.tabs.devices")}</SectionTitle>
              <SettingToggle
                title="Browser automation"
                desc="Агенты открывают страницы во встроенном браузере (или во внешнем, если Desktop control выкл.)"
                value={(d.devices?.browserAutomationEnabled as boolean) ?? false}
                onChange={(v) => update("devices", "browserAutomationEnabled", v)}
              />
              <SettingToggle
                title="Desktop control (dual mouse)"
                desc="Виртуальная ИИ-мышь и агент-браузер на вкладке «Устройства». Системный курсор не затрагивается."
                value={(d.devices?.desktopControlEnabled as boolean) ?? false}
                onChange={(v) => update("devices", "desktopControlEnabled", v)}
              />
            </>
          )}
          {tab === "injection" && <InjectionPanel d={d} u={update} />}
          {tab === "inference" && (
            <>
              <SectionTitle>{t("settings.inference.section")}</SectionTitle>
              <SettingSelect title="Backend" value={inf.defaultBackend as string} options={["gguf", "onnx", "safetensors", "pytorch", "tensorrt"]} onChange={(v) => update("inference", "defaultBackend", v)} />
              <SettingSlider title="Context" value={inf.contextLength as number} onChange={(v) => update("inference", "contextLength", v)} min={2048} max={131072} step={1024} />
              <SettingSlider title="Temperature" value={inf.temperature as number} onChange={(v) => update("inference", "temperature", v)} min={0} max={2} step={0.05} />
              <SettingNumber title="Top-K" value={inf.topK as number} onChange={(v) => update("inference", "topK", v)} />
              <SettingToggle title="Streaming" value={inf.streaming as boolean} onChange={(v) => update("inference", "streaming", v)} />
              <SettingToggle title="Flash Attention" value={inf.flashAttention as boolean} onChange={(v) => update("inference", "flashAttention", v)} />
              <SettingToggle title="Speculative Decoding" value={inf.speculativeDecoding as boolean} onChange={(v) => update("inference", "speculativeDecoding", v)} />
            </>
          )}
          {tab === "appearance" && (
            <>
              <SettingSelect title={t("settings.language")} value={draft.language} options={[{ v: "ru", l: "Русский" }, { v: "en", l: "English" }]} onChange={(v) => setDraft({ ...draft, language: v })} />
              <SettingSelect
                title={t("settings.appearance.theme")}
                desc={t("settings.appearance.themeDesc")}
                value={(ui.theme as string) || "dark"}
                options={THEME_OPTIONS.map((id) => ({ v: id, l: t(`settings.appearance.themes.${id}`) }))}
                onChange={(v) => void updateUiLive("theme", v)}
              />
              <SettingSlider title={t("settings.appearance.fontSize")} value={(ui.fontSize as number) ?? 14} onChange={(v) => void updateUiLive("fontSize", v)} min={12} max={24} />
              <SettingToggle title={t("settings.appearance.compact")} value={Boolean(ui.compactMode)} onChange={(v) => void updateUiLive("compactMode", v)} />
              <SettingToggle title={t("settings.appearance.tokenCounter")} value={Boolean(ui.showTokenCounter)} onChange={(v) => void updateUiLive("showTokenCounter", v)} />
              <SettingToggle title={t("settings.appearance.animations")} value={ui.animationsEnabled !== false} onChange={(v) => void updateUiLive("animationsEnabled", v)} />
            </>
          )}
          {tab === "advanced" && (
            <>
              <InnovationPanel d={d} u={update} />
              <SectionTitle>{t("settings.tabs.advanced")}</SectionTitle>
              <SettingToggle title="Debug mode" value={adv.debugMode as boolean} onChange={(v) => update("advanced", "debugMode", v)} />
              <SettingToggle title="Watchdog" value={adv.watchdogEnabled as boolean} onChange={(v) => update("advanced", "watchdogEnabled", v)} />
              <SettingSelect title="Sandbox" value={adv.sandboxLevel as string} options={["minimal", "standard", "strict", "maximum"]} onChange={(v) => update("advanced", "sandboxLevel", v)} />
              <SettingSelect title="Log level" value={adv.logLevel as string} options={["trace", "debug", "info", "warn", "error"]} onChange={(v) => update("advanced", "logLevel", v)} />
              <AgentGroupsEditor draft={draft} setDraft={setDraft} />
            </>
          )}
        </div>
      </div>
    </div>
  );
}

function AgentGroupsEditor({ draft, setDraft }: { draft: AppSettings; setDraft: (d: AppSettings) => void }) {
  const { t } = useTranslation();
  const groups = draft.agentGroups || [];
  const addGroup = () => {
    const g: AgentGroup = {
      id: `group-${Date.now()}`, name: "Research Team", enabled: true, orchestrationMode: "round_robin",
      members: [
        { id: "a1", name: "Coordinator", role: "coordinator", modelId: "default", permissions: { internet: false, camera: false, microphone: false, screen: false, stm: true, ltm: true, canDelegate: true } },
        { id: "a2", name: "Researcher", role: "researcher", modelId: "default", permissions: { internet: true, camera: false, microphone: false, screen: false, stm: true, ltm: true, canDelegate: false } },
      ],
      sharedMemory: true, maxRounds: 3, parallelExecution: false,
    };
    setDraft({ ...draft, agentGroups: [...groups, g] });
  };
  return (
    <div>
      <button className="btn btn-primary" onClick={addGroup} style={{ marginBottom: 16 }}>{t("settings.agents.addGroup")}</button>
      {groups.map((g, i) => (
        <div key={i} className="card" style={{ marginBottom: 10 }}>
          <strong>{g.name}</strong>
          <div style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 4 }}>{g.members.length} agents</div>
        </div>
      ))}
    </div>
  );
}

function ModelsEditor() {
  const { t } = useTranslation();
  const [path, setPath] = useState("");
  return (
    <div>
      <p className="field-hint" style={{ marginBottom: 16 }}>{t("settings.models.formats")}</p>
      <SettingText title={t("settings.models.localPath")} value={path} onChange={setPath} />
      <button className="btn btn-secondary" style={{ marginTop: 8 }} onClick={() => path && api.loadModel(path, path.split("/").pop() || "model")}>
        {t("settings.models.add")}
      </button>
    </div>
  );
}
