import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import { api, AgentGroup, AppSettings } from "../api/tauri";
import i18n from "../i18n";
import {
  SectionTitle, SettingNumber, SettingSelect, SettingSlider,
  SettingText, SettingToggle,
} from "./ui/SettingField";
import {
  InnovationPanel, InjectionPanel, MemoryPanel,
  NetworkPanel, PerformancePanel, SecurityPanel, SystemPanel,
} from "./settings/SettingsPanels";
import "./ui/ui.css";

const TABS = [
  { id: "innovation", icon: "🔮", highlight: true },
  { id: "system", icon: "⚡" },
  { id: "performance", icon: "🚀" },
  { id: "memory", icon: "💾" },
  { id: "network", icon: "🌐" },
  { id: "security", icon: "🛡️" },
  { id: "injection", icon: "✨", highlight: true },
  { id: "inference", icon: "🧠" },
  { id: "devices", icon: "📷" },
  { id: "agents", icon: "🤝" },
  { id: "models", icon: "📦" },
  { id: "ui", icon: "🎨" },
  { id: "advanced", icon: "🔧" },
] as const;

export function SettingsView() {
  const { t } = useTranslation();
  const { settings, setSettings } = useAppStore();
  const [tab, setTab] = useState("innovation");
  const [draft, setDraft] = useState(settings);
  const [saved, setSaved] = useState(false);
  const [search, setSearch] = useState("");

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

  const save = async () => {
    await api.updateSettings(draft as never);
    setSettings(draft);
    if (draft.language !== i18n.language) {
      i18n.changeLanguage(draft.language);
      localStorage.setItem("neuroforge-lang", draft.language);
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
          {tab === "system" && <SystemPanel d={d} u={update} />}
          {tab === "innovation" && <InnovationPanel d={d} u={update} />}
          {tab === "performance" && <PerformancePanel d={d} u={update} />}
          {tab === "security" && <SecurityPanel d={d} u={update} />}
          {tab === "memory" && <MemoryPanel d={d} u={update} />}
          {tab === "network" && <NetworkPanel d={d} u={update} />}
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
          {tab === "devices" && (
            <>
              <SectionTitle>{t("settings.devices.section")}</SectionTitle>
              <SettingToggle title={t("settings.devices.camera")} value={dev.cameraEnabled as boolean} onChange={(v) => update("devices", "cameraEnabled", v)} />
              <SettingToggle title={t("settings.devices.microphone")} value={dev.microphoneEnabled as boolean} onChange={(v) => update("devices", "microphoneEnabled", v)} />
              <SettingToggle title={t("settings.devices.screen")} value={dev.screenCaptureEnabled as boolean} onChange={(v) => update("devices", "screenCaptureEnabled", v)} />
              <SettingToggle title={t("settings.devices.virtualDisplay")} value={dev.virtualDisplayExtend as boolean} onChange={(v) => update("devices", "virtualDisplayExtend", v)} />
              <SettingNumber title={t("settings.devices.maxAttachment")} value={dev.maxAttachmentMb as number} onChange={(v) => update("devices", "maxAttachmentMb", v)} />
              <SettingToggle title={t("settings.devices.ocr")} value={dev.ocrOnImages as boolean} onChange={(v) => update("devices", "ocrOnImages", v)} />
            </>
          )}
          {tab === "agents" && <AgentGroupsEditor draft={draft} setDraft={setDraft} />}
          {tab === "models" && <ModelsEditor />}
          {tab === "ui" && (
            <>
              <SettingSelect title={t("settings.language")} value={draft.language} options={[{ v: "ru", l: "Русский" }, { v: "en", l: "English" }]} onChange={(v) => setDraft({ ...draft, language: v })} />
              <SettingSelect title="Theme" value={ui.theme as string} options={["dark", "light", "oled", "midnight", "aurora"]} onChange={(v) => update("ui", "theme", v)} />
              <SettingSlider title="Font size" value={ui.fontSize as number} onChange={(v) => update("ui", "fontSize", v)} min={12} max={20} />
              <SettingToggle title="Animations" value={ui.animationsEnabled as boolean} onChange={(v) => update("ui", "animationsEnabled", v)} />
            </>
          )}
          {tab === "advanced" && (
            <>
              <SettingToggle title="Debug mode" value={adv.debugMode as boolean} onChange={(v) => update("advanced", "debugMode", v)} />
              <SettingToggle title="Watchdog" value={adv.watchdogEnabled as boolean} onChange={(v) => update("advanced", "watchdogEnabled", v)} />
              <SettingSelect title="Sandbox" value={adv.sandboxLevel as string} options={["minimal", "standard", "strict", "maximum"]} onChange={(v) => update("advanced", "sandboxLevel", v)} />
              <SettingSelect title="Log level" value={adv.logLevel as string} options={["trace", "debug", "info", "warn", "error"]} onChange={(v) => update("advanced", "logLevel", v)} />
            </>
          )}
        </div>
      </div>
      <style>{`
        .settings { display: flex; flex-direction: column; height: 100%; }
        .settings-toolbar { display: flex; gap: 12px; padding: 12px 20px; border-bottom: 1px solid var(--border); background: var(--bg-elevated); }
        .settings-search { flex: 1; max-width: 360px; }
        .settings-toolbar-actions { display: flex; gap: 8px; align-items: center; margin-left: auto; }
        .settings-body { display: flex; flex: 1; overflow: hidden; }
        .settings-nav { width: 200px; border-right: 1px solid var(--border); padding: 8px; display: flex; flex-direction: column; gap: 2px; }
        .settings-nav-item { display: flex; align-items: center; gap: 8px; padding: 9px 12px; border-radius: var(--radius-sm); background: transparent; color: var(--text-secondary); font-size: 12px; font-weight: 500; text-align: left; border: none; }
        .settings-nav-item:hover { background: var(--bg-hover); color: var(--text); }
        .settings-nav-item.active { background: rgba(124,108,255,0.12); color: var(--accent-bright); font-weight: 600; }
        .settings-nav-item.highlight { border-left: 2px solid var(--accent-3); }
        .settings-content { flex: 1; padding: 20px 28px; max-width: 720px; }
      `}</style>
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
