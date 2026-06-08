import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import { api, AgentGroup, AppSettings } from "../api/tauri";
import i18n from "../i18n";

const TABS = ["system", "network", "memory", "inference", "devices", "injection", "agents", "models", "ui", "advanced"] as const;

export function SettingsView() {
  const { t } = useTranslation();
  const { settings, setSettings } = useAppStore();
  const [tab, setTab] = useState<string>("system");
  const [draft, setDraft] = useState(settings);
  const [saved, setSaved] = useState(false);

  if (!draft) return null;

  const update = (section: string, key: string, value: unknown) => {
    setDraft((d) => {
      if (!d) return d;
      const rec = d as unknown as Record<string, Record<string, unknown>>;
      const sec = { ...rec[section], [key]: value };
      return { ...d, [section]: sec } as typeof d;
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

  const sys = draft.system as Record<string, unknown>;
  const net = draft.network as Record<string, unknown>;
  const mem = draft.memory as Record<string, unknown>;
  const inj = draft.globalMessageInjection as Record<string, unknown>;
  const dev = draft.devices as Record<string, unknown>;
  const inf = draft.inference as Record<string, unknown>;
  const adv = draft.advanced as Record<string, unknown>;

  return (
    <div className="settings-view">
      <div className="settings-header">
        <h2>{t("settings.title")}</h2>
        <div className="settings-actions">
          {saved && <span className="badge badge-green">✓</span>}
          <button className="btn-danger" onClick={reset}>{t("settings.reset")}</button>
          <button className="btn-primary" onClick={save}>{t("settings.save")}</button>
        </div>
      </div>
      <div className="settings-body">
        <div className="settings-tabs scroll-y">
          {TABS.map((k) => (
            <button key={k} className={`stab ${tab === k ? "active" : ""}`} onClick={() => setTab(k)}>
              {t(`settings.tabs.${k}`)}
            </button>
          ))}
        </div>
        <div className="settings-panel scroll-y">
          {tab === "system" && (
            <>
              <NumberField label={t("settings.system.ramLimit")} value={sys.ramLimitMb as number} onChange={(v) => update("system", "ramLimitMb", v)} />
              <NumberField label={t("settings.system.ramSoft")} value={sys.ramSoftLimitPercent as number} onChange={(v) => update("system", "ramSoftLimitPercent", v)} />
              <TextField label={t("settings.system.cpuCores")} value={(sys.cpuCores as number[]).join(",")} onChange={(v) => update("system", "cpuCores", v.split(",").map(Number).filter((n) => !isNaN(n)))} />
              <SelectField label={t("settings.system.cpuAffinity")} value={sys.cpuAffinityMode as string} options={["auto", "manual", "performance", "efficiency"]} onChange={(v) => update("system", "cpuAffinityMode", v)} />
              <NumberField label={t("settings.system.gpuLayers")} value={sys.gpuLayers as number} onChange={(v) => update("system", "gpuLayers", v)} />
              <NumberField label={t("settings.system.gpuMemory")} value={sys.gpuMemoryMb as number} onChange={(v) => update("system", "gpuMemoryMb", v)} />
              <NumberField label={t("settings.system.threads")} value={sys.threadCount as number} onChange={(v) => update("system", "threadCount", v)} />
              <NumberField label={t("settings.system.batchSize")} value={sys.batchSize as number} onChange={(v) => update("system", "batchSize", v)} />
              <ToggleField label={t("settings.system.mmap")} value={sys.mmapEnabled as boolean} onChange={(v) => update("system", "mmapEnabled", v)} />
              <ToggleField label={t("settings.system.mlock")} value={sys.mlockEnabled as boolean} onChange={(v) => update("system", "mlockEnabled", v)} />
              <NumberField label={t("settings.system.numa")} value={sys.numaNode as number} onChange={(v) => update("system", "numaNode", v)} />
              <SelectField label={t("settings.system.priority")} value={sys.processPriority as string} options={["low", "normal", "high", "realtime"]} onChange={(v) => update("system", "processPriority", v)} />
              <SelectField label={t("settings.system.swap")} value={sys.swapUsage as string} options={["none", "minimal", "aggressive"]} onChange={(v) => update("system", "swapUsage", v)} />
              <NumberField label={t("settings.system.diskCache")} value={sys.diskCacheMb as number} onChange={(v) => update("system", "diskCacheMb", v)} />
              <NumberField label={t("settings.system.autoGc")} value={sys.autoGcIntervalSec as number} onChange={(v) => update("system", "autoGcIntervalSec", v)} />
              <SelectField label={t("settings.system.oomPolicy")} value={sys.oomPolicy as string} options={["kill", "graceful_degrade", "swap"]} onChange={(v) => update("system", "oomPolicy", v)} />
            </>
          )}
          {tab === "network" && (
            <>
              <SelectField label={t("settings.network.isolation")} value={net.isolationMode as string} options={[
                { v: "full", l: t("settings.network.isolationFull") },
                { v: "api_only", l: t("settings.network.isolationApi") },
                { v: "none", l: t("settings.network.isolationNone") },
              ]} onChange={(v) => update("network", "isolationMode", v)} />
              <ToggleField label={t("settings.network.allowInternet")} value={net.allowInternet as boolean} onChange={(v) => update("network", "allowInternet", v)} />
              <TextAreaField label={t("settings.network.apiEndpoints")} value={(net.apiOnlyEndpoints as string[]).join("\n")} onChange={(v) => update("network", "apiOnlyEndpoints", v.split("\n").filter(Boolean))} />
              <TextField label={t("settings.network.proxy")} value={net.proxyUrl as string} onChange={(v) => update("network", "proxyUrl", v)} />
              <ToggleField label={t("settings.network.doh")} value={net.dnsOverHttps as boolean} onChange={(v) => update("network", "dnsOverHttps", v)} />
              <NumberField label={t("settings.network.timeout")} value={net.requestTimeoutSec as number} onChange={(v) => update("network", "requestTimeoutSec", v)} />
              <NumberField label={t("settings.network.maxRequests")} value={net.maxConcurrentRequests as number} onChange={(v) => update("network", "maxConcurrentRequests", v)} />
              <ToggleField label={t("settings.network.logRequests")} value={net.logAllRequests as boolean} onChange={(v) => update("network", "logAllRequests", v)} />
              <ToggleField label={t("settings.network.blockPrivate")} value={net.blockPrivateIps as boolean} onChange={(v) => update("network", "blockPrivateIps", v)} />
              <ToggleField label={t("settings.network.tls")} value={net.tlsVerify as boolean} onChange={(v) => update("network", "tlsVerify", v)} />
              <TextField label={t("settings.network.hfMirror")} value={net.huggingfaceMirror as string} onChange={(v) => update("network", "huggingfaceMirror", v)} />
            </>
          )}
          {tab === "memory" && (
            <>
              <ToggleField label={t("settings.memory.stmEnabled")} value={mem.stmEnabled as boolean} onChange={(v) => update("memory", "stmEnabled", v)} />
              <NumberField label={t("settings.memory.stmMaxTokens")} value={mem.stmMaxTokens as number} onChange={(v) => update("memory", "stmMaxTokens", v)} />
              <NumberField label={t("settings.memory.stmTtl")} value={mem.stmTtlMinutes as number} onChange={(v) => update("memory", "stmTtlMinutes", v)} />
              <ToggleField label={t("settings.memory.ltmEnabled")} value={mem.ltmEnabled as boolean} onChange={(v) => update("memory", "ltmEnabled", v)} />
              <NumberField label={t("settings.memory.ltmMaxEntries")} value={mem.ltmMaxEntries as number} onChange={(v) => update("memory", "ltmMaxEntries", v)} />
              <SelectField label={t("settings.memory.ltmPersistence")} value={mem.ltmPersistence as string} options={["sqlite", "json", "rocksdb"]} onChange={(v) => update("memory", "ltmPersistence", v)} />
              <SelectField label={t("settings.memory.transferPolicy")} value={mem.transferPolicy as string} options={["explicit_approval", "auto", "disabled"]} onChange={(v) => update("memory", "transferPolicy", v)} />
              <ToggleField label={t("settings.memory.crossChat")} value={mem.crossChatTransfer as boolean} onChange={(v) => update("memory", "crossChatTransfer", v)} />
              <ToggleField label={t("settings.memory.crossModel")} value={mem.crossModelTransfer as boolean} onChange={(v) => update("memory", "crossModelTransfer", v)} />
              <ToggleField label={t("settings.memory.autoConsolidate")} value={mem.autoConsolidate as boolean} onChange={(v) => update("memory", "autoConsolidate", v)} />
              <ToggleField label={t("settings.memory.encryption")} value={mem.memoryEncryption as boolean} onChange={(v) => update("memory", "memoryEncryption", v)} />
              <NumberField label={t("settings.memory.recallTopK")} value={mem.recallTopK as number} onChange={(v) => update("memory", "recallTopK", v)} />
              <ToggleField label={t("settings.memory.semanticSearch")} value={mem.semanticSearch as boolean} onChange={(v) => update("memory", "semanticSearch", v)} />
              <NumberField label={t("settings.memory.decayRate")} value={mem.decayRate as number} step={0.01} onChange={(v) => update("memory", "decayRate", v)} />
            </>
          )}
          {tab === "injection" && (
            <div className="injection-section">
              <div className="innovation-badge">✨ {t("settings.injection.title")}</div>
              <p className="injection-desc">{t("settings.injection.desc")}</p>
              <ToggleField label={t("settings.injection.enabled")} value={inj.enabled as boolean} onChange={(v) => update("globalMessageInjection", "enabled", v)} />
              <TextAreaField label={t("settings.injection.systemPrefix")} value={inj.systemPrefix as string} onChange={(v) => update("globalMessageInjection", "systemPrefix", v)} />
              <TextAreaField label={t("settings.injection.userSuffix")} value={inj.userSuffix as string} onChange={(v) => update("globalMessageInjection", "userSuffix", v)} />
              <TextAreaField label={t("settings.injection.hiddenContext")} value={inj.hiddenContext as string} onChange={(v) => update("globalMessageInjection", "hiddenContext", v)} />
              <ToggleField label={t("settings.injection.injectMemory")} value={inj.injectMemorySummary as boolean} onChange={(v) => update("globalMessageInjection", "injectMemorySummary", v)} />
              <ToggleField label={t("settings.injection.injectDevice")} value={inj.injectDeviceState as boolean} onChange={(v) => update("globalMessageInjection", "injectDeviceState", v)} />
              <ToggleField label={t("settings.injection.injectTime")} value={inj.injectTimestamp as boolean} onChange={(v) => update("globalMessageInjection", "injectTimestamp", v)} />
              <ToggleField label={t("settings.injection.injectLocale")} value={inj.injectLocale as boolean} onChange={(v) => update("globalMessageInjection", "injectLocale", v)} />
            </div>
          )}
          {tab === "devices" && (
            <>
              <ToggleField label={t("settings.devices.camera")} value={dev.cameraEnabled as boolean} onChange={(v) => update("devices", "cameraEnabled", v)} />
              <ToggleField label={t("settings.devices.microphone")} value={dev.microphoneEnabled as boolean} onChange={(v) => update("devices", "microphoneEnabled", v)} />
              <ToggleField label={t("settings.devices.screen")} value={dev.screenCaptureEnabled as boolean} onChange={(v) => update("devices", "screenCaptureEnabled", v)} />
              <ToggleField label={t("settings.devices.virtualDisplay")} value={dev.virtualDisplayExtend as boolean} onChange={(v) => update("devices", "virtualDisplayExtend", v)} />
              <TextField label={t("settings.devices.resolution")} value={dev.virtualDisplayResolution as string} onChange={(v) => update("devices", "virtualDisplayResolution", v)} />
              <NumberField label={t("settings.devices.maxAttachment")} value={dev.maxAttachmentMb as number} onChange={(v) => update("devices", "maxAttachmentMb", v)} />
              <ToggleField label={t("settings.devices.ocr")} value={dev.ocrOnImages as boolean} onChange={(v) => update("devices", "ocrOnImages", v)} />
              <ToggleField label={t("settings.devices.transcribe")} value={dev.transcribeAudio as boolean} onChange={(v) => update("devices", "transcribeAudio", v)} />
              <NumberField label={t("settings.devices.frameRate")} value={dev.frameRate as number} onChange={(v) => update("devices", "frameRate", v)} />
            </>
          )}
          {tab === "inference" && (
            <>
              <SelectField label="Backend" value={inf.defaultBackend as string} options={["gguf", "onnx", "safetensors", "pytorch"]} onChange={(v) => update("inference", "defaultBackend", v)} />
              <NumberField label="Context length" value={inf.contextLength as number} onChange={(v) => update("inference", "contextLength", v)} />
              <NumberField label="Temperature" value={inf.temperature as number} step={0.1} onChange={(v) => update("inference", "temperature", v)} />
              <NumberField label="Top-P" value={inf.topP as number} step={0.05} onChange={(v) => update("inference", "topP", v)} />
              <NumberField label="Top-K" value={inf.topK as number} onChange={(v) => update("inference", "topK", v)} />
              <ToggleField label="Streaming" value={inf.streaming as boolean} onChange={(v) => update("inference", "streaming", v)} />
              <ToggleField label="Flash attention" value={inf.flashAttention as boolean} onChange={(v) => update("inference", "flashAttention", v)} />
              <p className="formats-info">{t("settings.models.formats")}</p>
            </>
          )}
          {tab === "agents" && <AgentGroupsEditor draft={draft} setDraft={setDraft} />}
          {tab === "models" && <ModelsEditor draft={draft} setDraft={setDraft} />}
          {tab === "ui" && (
            <>
              <SelectField label={t("settings.language")} value={draft.language} options={[
                { v: "ru", l: "Русский" },
                { v: "en", l: "English" },
              ]} onChange={(v) => setDraft({ ...draft, language: v })} />
              <SelectField label="Theme" value={(draft.ui as Record<string, unknown>).theme as string} options={["dark", "light", "oled"]} onChange={(v) => update("ui", "theme", v)} />
              <NumberField label="Font size" value={(draft.ui as Record<string, unknown>).fontSize as number} onChange={(v) => update("ui", "fontSize", v)} />
              <ToggleField label="Compact mode" value={(draft.ui as Record<string, unknown>).compactMode as boolean} onChange={(v) => update("ui", "compactMode", v)} />
              <ToggleField label="Animations" value={(draft.ui as Record<string, unknown>).animationsEnabled as boolean} onChange={(v) => update("ui", "animationsEnabled", v)} />
              <ToggleField label="High contrast" value={(draft.ui as Record<string, unknown>).highContrast as boolean} onChange={(v) => update("ui", "highContrast", v)} />
            </>
          )}
          {tab === "advanced" && (
            <>
              <ToggleField label="Debug mode" value={adv.debugMode as boolean} onChange={(v) => update("advanced", "debugMode", v)} />
              <ToggleField label="Watchdog" value={adv.watchdogEnabled as boolean} onChange={(v) => update("advanced", "watchdogEnabled", v)} />
              <ToggleField label="Auto-restart on crash" value={adv.autoRestartOnCrash as boolean} onChange={(v) => update("advanced", "autoRestartOnCrash", v)} />
              <SelectField label="Sandbox level" value={adv.sandboxLevel as string} options={["minimal", "standard", "strict"]} onChange={(v) => update("advanced", "sandboxLevel", v)} />
              <SelectField label="Log level" value={adv.logLevel as string} options={["trace", "debug", "info", "warn", "error"]} onChange={(v) => update("advanced", "logLevel", v)} />
              <ToggleField label="Experimental features" value={adv.experimentalFeatures as boolean} onChange={(v) => update("advanced", "experimentalFeatures", v)} />
            </>
          )}
        </div>
      </div>
      <style>{`
        .settings-view { display: flex; flex-direction: column; height: 100%; }
        .settings-header { display: flex; align-items: center; padding: 12px 16px; border-bottom: 1px solid var(--border); gap: 12px; }
        .settings-header h2 { flex: 1; }
        .settings-actions { display: flex; gap: 8px; align-items: center; }
        .settings-body { display: flex; flex: 1; overflow: hidden; }
        .settings-tabs { width: 180px; border-right: 1px solid var(--border); padding: 8px; display: flex; flex-direction: column; gap: 2px; }
        .stab { text-align: left; padding: 8px 10px; border-radius: 6px; background: transparent; color: var(--text2); border: none; font-size: 12px; }
        .stab.active { background: var(--bg3); color: var(--accent2); font-weight: 600; }
        .settings-panel { flex: 1; padding: 16px 20px; max-width: 600px; }
        .innovation-badge { background: linear-gradient(135deg, #312e81, #4c1d95); padding: 10px 14px; border-radius: var(--radius); font-weight: 600; margin-bottom: 12px; }
        .injection-desc { color: var(--text2); margin-bottom: 16px; line-height: 1.5; }
        .formats-info { color: var(--text2); font-size: 12px; margin-top: 12px; }
      `}</style>
    </div>
  );
}

function NumberField({ label, value, onChange, step = 1 }: { label: string; value: number; onChange: (v: number) => void; step?: number }) {
  return (
    <div className="field">
      <label className="label">{label}</label>
      <input type="number" value={value} step={step} onChange={(e) => onChange(Number(e.target.value))} style={{ width: "100%" }} />
    </div>
  );
}

function TextField({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return (
    <div className="field">
      <label className="label">{label}</label>
      <input type="text" value={value} onChange={(e) => onChange(e.target.value)} style={{ width: "100%" }} />
    </div>
  );
}

function TextAreaField({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return (
    <div className="field">
      <label className="label">{label}</label>
      <textarea value={value} onChange={(e) => onChange(e.target.value)} rows={3} style={{ width: "100%" }} />
    </div>
  );
}

function ToggleField({ label, value, onChange }: { label: string; value: boolean; onChange: (v: boolean) => void }) {
  return (
    <div className="toggle-row">
      <label>{label}</label>
      <input type="checkbox" checked={value} onChange={(e) => onChange(e.target.checked)} />
    </div>
  );
}

function SelectField({ label, value, options, onChange }: {
  label: string; value: string;
  options: string[] | { v: string; l: string }[];
  onChange: (v: string) => void;
}) {
  const opts = options.map((o) => (typeof o === "string" ? { v: o, l: o } : o));
  return (
    <div className="field">
      <label className="label">{label}</label>
      <select value={value} onChange={(e) => onChange(e.target.value)} style={{ width: "100%" }}>
        {opts.map((o) => <option key={o.v} value={o.v}>{o.l}</option>)}
      </select>
    </div>
  );
}

function AgentGroupsEditor({ draft, setDraft }: { draft: AppSettings; setDraft: (d: AppSettings) => void }) {
  const { t } = useTranslation();
  const groups = draft.agentGroups || [];

  const addGroup = () => {
    const g: AgentGroup = {
      id: `group-${Date.now()}`,
      name: "Research Team",
      enabled: true,
      orchestrationMode: "round_robin",
      members: [
        { id: "a1", name: "Coordinator", role: "coordinator", modelId: "default", permissions: { internet: false, camera: false, microphone: false, screen: false, stm: true, ltm: true, canDelegate: true } },
        { id: "a2", name: "Researcher", role: "researcher", modelId: "default", permissions: { internet: true, camera: false, microphone: false, screen: false, stm: true, ltm: true, canDelegate: false } },
        { id: "a3", name: "Executor", role: "executor", modelId: "default", permissions: { internet: false, camera: true, microphone: true, screen: true, stm: true, ltm: false, canDelegate: false } },
      ],
      sharedMemory: true,
      maxRounds: 3,
      parallelExecution: false,
    };
    setDraft({ ...draft, agentGroups: [...groups, g] });
  };

  return (
    <div>
      <button className="btn-primary" onClick={addGroup} style={{ marginBottom: 16 }}>{t("settings.agents.addGroup")}</button>
      {groups.map((g, i) => (
        <div key={i} className="card" style={{ marginBottom: 12 }}>
          <strong>{g.name}</strong>
          <div style={{ fontSize: 12, color: "var(--text2)", marginTop: 4 }}>
            {g.members.length} agents | {g.orchestrationMode} | {g.maxRounds} rounds
          </div>
        </div>
      ))}
    </div>
  );
}

function ModelsEditor({ draft, setDraft }: { draft: AppSettings; setDraft: (d: AppSettings) => void }) {
  const { t } = useTranslation();
  const [path, setPath] = useState("");
  const [repo, setRepo] = useState("");
  const [models, setModels] = useState<{ id: string; name: string; format: string }[]>([]);

  const loadLocal = async () => {
    if (!path) return;
    const m = await api.loadModel(path, path.split("/").pop() || "model") as { id: string; name: string; format: string };
    setModels((ms) => [...ms, m]);
    const customs = [...((draft.customModels || []) as unknown[]), { id: m.id, name: m.name, path, format: m.format, backend: "local", parameters: {} }];
    setDraft({ ...draft, customModels: customs } as AppSettings);
  };

  const loadHf = async () => {
    if (!repo) return;
    const m = await api.loadHuggingfaceModel(repo) as { id: string; name: string; format: string };
    setModels((ms) => [...ms, m]);
  };

  return (
    <div>
      <p className="formats-info">{t("settings.models.formats")}</p>
      <TextField label={t("settings.models.localPath")} value={path} onChange={setPath} />
      <button className="btn-secondary" onClick={loadLocal} style={{ marginBottom: 12 }}>{t("settings.models.add")}</button>
      <TextField label={t("settings.models.hfRepo")} value={repo} onChange={setRepo} />
      <button className="btn-secondary" onClick={loadHf}>{t("settings.models.add")} (HF)</button>
      {models.map((m) => (
        <div key={m.id} className="card" style={{ marginTop: 8 }}>{m.name} <span className="badge badge-blue">{m.format}</span></div>
      ))}
    </div>
  );
}
