import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../../store/appStore";
import { AgentGroup, AgentMember } from "../../api/tauri";
import { Tooltip } from "../ui/Tooltip";
import { EmptyState } from "../ui/EmptyState";
import {
  AGENT_ROLES, AGENT_TOOLS, ORCHESTRATION_STRATEGIES,
  CONFLICT_MODES, TRIGGER_CONDITIONS,
} from "../../constants/agents";
import { OrchestrationMonitor } from "./OrchestrationMonitor";
import { AgentSystemsGuide } from "./AgentSystemsGuide";
import { ModelSelect } from "../models/ModelSelect";
import { useModels } from "../../hooks/useModels";
import { applyRoleDefaults, roleDefaultPrompt } from "../../constants/rolePrompts";
import { api } from "../../api/tauri";

function cloneMember(member: AgentMember, nameSuffix?: string): AgentMember {
  return {
    ...member,
    id: `agent-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
    name: nameSuffix ? `${member.name} ${nameSuffix}` : member.name,
    permissions: { ...member.permissions },
    resources: member.resources
      ? { ...member.resources, cpuCores: [...member.resources.cpuCores] }
      : member.resources,
    tools: [...(member.tools || [])],
  };
}

function cloneGroup(group: AgentGroup, t: (k: string) => string): AgentGroup {
  return {
    ...group,
    id: `group-${Date.now()}`,
    name: `${group.name} (${t("agents.copySuffix")})`,
    members: group.members.map((m) => cloneMember(m)),
  };
}

function newMember(lang: "ru" | "en"): AgentMember {
  return {
    id: `agent-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
    name: "Agent",
    role: "worker",
    modelId: "default",
    permissions: {
      internet: false, camera: false, microphone: false, screen: false,
      stm: true, ltm: true, canDelegate: false, files: false, tools: true,
      veto: false, sharedMemory: true,
    },
    resources: {
      ramLimitMb: 2048, cpuCores: [0, 1], maxTokens: 2048,
      temperature: 0.7, executionOrder: 0,
    },
    tools: ["memory_query", "calculator"],
    trigger: "always",
    triggerKeyword: "",
    systemPrompt: roleDefaultPrompt("worker", lang),
    systemPromptCustomized: false,
  } as AgentMember & { systemPromptCustomized?: boolean };
}

export function AgentStudio() {
  const { t, i18n } = useTranslation();
  const lang = i18n.language === "ru" ? "ru" : "en";
  const { settings, setSettings, selectedGroupId, setSelectedGroupId, pushMonitorEvent } = useAppStore();
  const { models } = useModels();
  const modelLabel = (id: string) => models.find((m) => m.id === id)?.name ?? id;
  const [tab, setTab] = useState<"groups" | "editor" | "monitor">("groups");
  const [runPrompt, setRunPrompt] = useState("");
  const [running, setRunning] = useState(false);
  const [guideOpen, setGuideOpen] = useState(false);
  const [focusMemberId, setFocusMemberId] = useState<string | null>(null);

  const groups = (settings?.agentGroups || []) as (AgentGroup & {
    conflictMode?: string; timeoutSec?: number; feedbackLoops?: boolean;
    taskDecomposition?: boolean; trigger?: string;
  })[];

  const selected = groups.find((g) => g.id === selectedGroupId) || groups[0];

  const saveGroups = async (next: AgentGroup[]) => {
    if (!settings) return;
    const updated = { ...settings, agentGroups: next };
    await api.updateSettings(updated as never);
    setSettings(updated);
  };

  const addGroup = () => {
    const g = {
      id: `group-${Date.now()}`,
      name: t("agents.newGroup"),
      enabled: true,
      orchestrationMode: "sequential",
      members: [newMember(lang), newMember(lang)],
      sharedMemory: true,
      maxRounds: 5,
      parallelExecution: false,
      consensusThreshold: 0.7,
      conflictMode: "consensus",
      timeoutSec: 120,
      feedbackLoops: true,
      taskDecomposition: true,
    } as AgentGroup;
    saveGroups([...groups, g]);
    setSelectedGroupId(g.id);
    setTab("editor");
  };

  const updateGroup = (patch: Partial<AgentGroup>) => {
    if (!selected) return;
    saveGroups(groups.map((g) => (g.id === selected.id ? { ...g, ...patch } : g)));
  };

  const updateMember = (memberId: string, patch: Partial<AgentMember>) => {
    if (!selected) return;
    updateGroup({
      members: selected.members.map((m) =>
        m.id === memberId ? { ...m, ...patch } : m
      ),
    });
  };

  const deleteGroup = (groupId: string) => {
    if (!confirm(t("agents.deleteGroupConfirm"))) return;
    const next = groups.filter((g) => g.id !== groupId);
    saveGroups(next);
    if (selectedGroupId === groupId) {
      setSelectedGroupId(next[0]?.id ?? null);
    }
  };

  const duplicateGroup = (group: AgentGroup) => {
    const copy = cloneGroup(group, t);
    saveGroups([...groups, copy]);
    setSelectedGroupId(copy.id);
    setTab("editor");
  };

  const copyGroupJson = async (group: AgentGroup) => {
    await navigator.clipboard.writeText(JSON.stringify(group, null, 2));
    pushMonitorEvent({
      id: `copy-${Date.now()}`,
      timestamp: new Date().toISOString(),
      type: "info",
      message: t("agents.copiedGroup"),
      status: "ok",
    });
  };

  const pasteGroupJson = async () => {
    try {
      const raw = await navigator.clipboard.readText();
      const parsed = JSON.parse(raw) as AgentGroup;
      if (!parsed.members || !Array.isArray(parsed.members)) throw new Error("invalid");
      const imported = cloneGroup({ ...parsed, name: parsed.name || t("agents.newGroup") }, t);
      saveGroups([...groups, imported]);
      setSelectedGroupId(imported.id);
      setTab("editor");
    } catch {
      alert(t("agents.pasteGroupError"));
    }
  };

  const runTeam = async () => {
    if (!selected || !runPrompt) return;
    setRunning(true);
    pushMonitorEvent({
      id: `e-${Date.now()}`, timestamp: new Date().toISOString(),
      type: "start", message: `${t("agents.runTeam")}: ${selected.name}`, status: "running",
    });
    setTab("monitor");
    try {
      const result = await api.runAgentTeam(selected.id, runPrompt);
      result.rounds.forEach((r) => {
        r.messages.forEach((m) => {
          pushMonitorEvent({
            id: `e-${Date.now()}-${m.agentName}`,
            timestamp: new Date().toISOString(),
            type: "agent", agentName: m.agentName,
            message: `${m.content.slice(0, 160)}${m.toolsUsed?.length ? ` [${m.toolsUsed.join(", ")}]` : ""}`,
            status: "ok",
          });
        });
      });
      pushMonitorEvent({
        id: `e-done-${Date.now()}`, timestamp: new Date().toISOString(),
        type: "done", message: result.status, status: "ok",
      });
    } catch (e) {
      pushMonitorEvent({
        id: `e-err-${Date.now()}`, timestamp: new Date().toISOString(),
        type: "error", message: String(e), status: "error",
      });
    }
    setRunning(false);
  };

  return (
    <div className="agent-studio" style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div className="agent-studio-toolbar">
        <div className="m3-tabs">
          <button type="button" className={`m3-tab ${tab === "groups" ? "active" : ""}`} onClick={() => setTab("groups")}>{t("agents.tabs.groups")}</button>
          <button type="button" className={`m3-tab ${tab === "editor" ? "active" : ""}`} onClick={() => setTab("editor")}>{t("agents.tabs.editor")}</button>
          <button type="button" className={`m3-tab ${tab === "monitor" ? "active" : ""}`} onClick={() => setTab("monitor")}>{t("agents.tabs.monitor")}</button>
        </div>
        <button
          type="button"
          className={`m3-tonal-btn agent-help-btn${guideOpen ? " active" : ""}`}
          onClick={() => setGuideOpen((v) => !v)}
        >
          ? {t("agents.guide.title")}
        </button>
      </div>

      <div className="scroll" style={{ flex: 1, padding: 16 }}>
        {guideOpen && (
          <AgentSystemsGuide
            open={guideOpen}
            onOpenChange={setGuideOpen}
            selectedMember={selected?.members.find((m) => m.id === focusMemberId)}
            onApplyRole={(roleId) => {
              if (!selected || !focusMemberId) return;
              const m = selected.members.find((x) => x.id === focusMemberId);
              if (!m) return;
              updateMember(focusMemberId, {
                ...applyRoleDefaults(m as AgentMember & { systemPromptCustomized?: boolean }, roleId, lang),
                systemPromptCustomized: false,
              });
            }}
          />
        )}

        {tab === "groups" && (
          <div>
            <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 16, gap: 12, flexWrap: "wrap" }}>
              <div>
                <h3 style={{ fontSize: 16, marginBottom: 4 }}>{t("agents.title")}</h3>
                <p style={{ fontSize: 13, color: "var(--m3-on-surface-variant)" }}>{t("agents.subtitle")}</p>
              </div>
              <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                <button type="button" className="m3-outlined-btn" onClick={pasteGroupJson}>{t("agents.pasteGroup")}</button>
                <button type="button" className="m3-filled-btn" onClick={addGroup}>+ {t("agents.addGroup")}</button>
              </div>
            </div>
            <div className="agent-grid">
              {groups.length === 0 && (
                <EmptyState
                  icon="🤝"
                  title={t("agents.emptyTitle")}
                  description={t("agents.emptyDesc")}
                  action={
                    <button type="button" className="m3-filled-btn" onClick={addGroup}>
                      + {t("agents.addGroup")}
                    </button>
                  }
                />
              )}
              {groups.map((g) => (
                <div key={g.id} className="m3-card agent-member-card agent-group-card">
                  <div
                    className="agent-group-card-body"
                    onClick={() => { setSelectedGroupId(g.id); setTab("editor"); }}
                    onKeyDown={(e) => e.key === "Enter" && (setSelectedGroupId(g.id), setTab("editor"))}
                    role="button"
                    tabIndex={0}
                  >
                  <h4>{g.name}</h4>
                  <p style={{ fontSize: 12, color: "var(--m3-outline)", marginBottom: 8 }}>
                    {ORCHESTRATION_STRATEGIES.find((s) => s.id === g.orchestrationMode)?.[lang]} · {g.members.length} {t("agents.members")}
                  </p>
                  <div style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
                    {g.members.slice(0, 4).map((m) => (
                      <span key={m.id} className="m3-chip" title={modelLabel(m.modelId)}>
                        {AGENT_ROLES.find((r) => r.id === m.role)?.[lang] || m.role} · {modelLabel(m.modelId).slice(0, 14)}
                      </span>
                    ))}
                  </div>
                  </div>
                  <div className="agent-group-card-actions" onClick={(e) => e.stopPropagation()}>
                    <Tooltip text={t("agents.copyGroupTip")}>
                      <button type="button" className="m3-tonal-btn sm" onClick={() => copyGroupJson(g)}>📋</button>
                    </Tooltip>
                    <Tooltip text={t("agents.duplicateGroupTip")}>
                      <button type="button" className="m3-tonal-btn sm" onClick={() => duplicateGroup(g)}>⧉</button>
                    </Tooltip>
                    <Tooltip text={t("agents.deleteGroupTip")}>
                      <button type="button" className="m3-tonal-btn sm danger" onClick={() => deleteGroup(g.id)}>✕</button>
                    </Tooltip>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {tab === "editor" && selected && (
          <div style={{ maxWidth: 900 }}>
            <div className="agent-editor-toolbar">
              <button type="button" className="m3-outlined-btn" onClick={() => copyGroupJson(selected)}>{t("agents.copyGroup")}</button>
              <button type="button" className="m3-outlined-btn" onClick={() => duplicateGroup(selected)}>{t("agents.duplicateGroup")}</button>
              <button type="button" className="m3-outlined-btn danger" onClick={() => deleteGroup(selected.id)}>{t("agents.deleteGroup")}</button>
            </div>
            <div className="form-row">
              <label className="form-label">{t("agents.groupName")}</label>
              <input className="m3-input" value={selected.name} onChange={(e) => updateGroup({ name: e.target.value })} />
            </div>

            <h4 style={{ margin: "20px 0 12px", fontSize: 14 }}>{t("agents.groupSettings")}</h4>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
              <div className="form-row">
                <Tooltip text={t("agents.tip.orchestration")}>
                  <label className="form-label">{t("agents.orchestration")} ⓘ</label>
                </Tooltip>
                <select className="m3-input" value={selected.orchestrationMode} onChange={(e) => updateGroup({ orchestrationMode: e.target.value })}>
                  {ORCHESTRATION_STRATEGIES.map((s) => <option key={s.id} value={s.id}>{s[lang]}</option>)}
                </select>
              </div>
              <div className="form-row">
                <Tooltip text={t("agents.tip.conflict")}>
                  <label className="form-label">{t("agents.conflictMode")} ⓘ</label>
                </Tooltip>
                <select className="m3-input" value={(selected as { conflictMode?: string }).conflictMode || "consensus"} onChange={(e) => updateGroup({ conflictMode: e.target.value } as Partial<AgentGroup>)}>
                  {CONFLICT_MODES.map((c) => <option key={c.id} value={c.id}>{c[lang]}</option>)}
                </select>
              </div>
              <div className="form-row">
                <label className="form-label">{t("agents.maxRounds")}</label>
                <input type="number" className="m3-input" value={selected.maxRounds} onChange={(e) => updateGroup({ maxRounds: Number(e.target.value) })} />
              </div>
              <div className="form-row">
                <label className="form-label">{t("agents.timeout")}</label>
                <input type="number" className="m3-input" value={(selected as { timeoutSec?: number }).timeoutSec ?? 120} onChange={(e) => updateGroup({ timeoutSec: Number(e.target.value) } as Partial<AgentGroup>)} />
              </div>
              <div className="form-row">
                <label className="form-label">{t("agents.consensus")}</label>
                <input type="number" step={0.05} className="m3-input" value={selected.consensusThreshold ?? 0.7} onChange={(e) => updateGroup({ consensusThreshold: Number(e.target.value) })} />
              </div>
              <div className="form-row" style={{ display: "flex", gap: 16, alignItems: "center" }}>
                <label style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 13 }}>
                  <input type="checkbox" checked={(selected as { feedbackLoops?: boolean }).feedbackLoops ?? true}
                    onChange={(e) => updateGroup({ feedbackLoops: e.target.checked } as Partial<AgentGroup>)} />
                  {t("agents.feedbackLoops")}
                </label>
                <label style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 13 }}>
                  <input type="checkbox" checked={(selected as { taskDecomposition?: boolean }).taskDecomposition ?? true}
                    onChange={(e) => updateGroup({ taskDecomposition: e.target.checked } as Partial<AgentGroup>)} />
                  {t("agents.taskDecomposition")}
                </label>
              </div>
            </div>

            <h4 style={{ margin: "24px 0 12px", fontSize: 14 }}>{t("agents.membersTitle")}</h4>
            {selected.members.map((m) => (
              <MemberEditor key={m.id} member={m} lang={lang} t={t}
                onFocusMember={() => setFocusMemberId(m.id)}
                onOpenHelp={() => { setFocusMemberId(m.id); setGuideOpen(true); }}
                onChange={(p) => updateMember(m.id, p)}
                onRemove={() => updateGroup({ members: selected.members.filter((x) => x.id !== m.id) })}
                onDuplicate={() => updateGroup({
                  members: [...selected.members, cloneMember(m, t("agents.copySuffix"))],
                })}
                onCopyJson={() => navigator.clipboard.writeText(JSON.stringify(m, null, 2))}
              />
            ))}
            <button type="button" className="m3-outlined-btn" style={{ marginTop: 8 }} onClick={() => updateGroup({ members: [...selected.members, newMember(lang)] })}>
              + {t("agents.addMember")}
            </button>

            <div style={{ marginTop: 24, padding: 16, background: "var(--m3-surface-container-highest)", borderRadius: 12 }}>
              <label className="form-label">{t("agents.prompt")}</label>
              <textarea className="m3-input" rows={3} value={runPrompt} onChange={(e) => setRunPrompt(e.target.value)} />
              <div style={{ display: "flex", gap: 8, marginTop: 10, flexWrap: "wrap" }}>
                <button type="button" className="m3-filled-btn" onClick={runTeam} disabled={running}>
                  {running ? "..." : t("agents.runTeam")}
                </button>
                {running && (
                  <button type="button" className="m3-outlined-btn danger" onClick={() => api.stopAgentTeam().catch(() => {})}>
                    {t("agents.stopTeam")}
                  </button>
                )}
              </div>
            </div>
          </div>
        )}

        {tab === "monitor" && <OrchestrationMonitor />}
      </div>
    </div>
  );
}

function MemberEditor({
  member, lang, t, onChange, onRemove, onDuplicate, onCopyJson, onFocusMember, onOpenHelp,
}: {
  member: AgentMember & { tools?: string[]; trigger?: string; triggerKeyword?: string; systemPrompt?: string; systemPromptCustomized?: boolean; resources?: Record<string, unknown> };
  lang: "ru" | "en";
  t: (k: string) => string;
  onChange: (p: Partial<AgentMember>) => void;
  onRemove: () => void;
  onDuplicate: () => void;
  onCopyJson: () => void;
  onFocusMember: () => void;
  onOpenHelp: () => void;
}) {
  const [open, setOpen] = useState(true);
  const res = member.resources || { ramLimitMb: 2048, cpuCores: [0, 1], maxTokens: 2048, temperature: 0.7, executionOrder: 0 };

  return (
    <div className="m3-card" style={{ marginBottom: 12 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer" }} onClick={() => { setOpen(!open); onFocusMember(); }}>
        <span>{open ? "▼" : "▶"}</span>
        <input className="m3-input" style={{ flex: 1, padding: "6px 10px" }} value={member.name}
          onChange={(e) => onChange({ name: e.target.value })} onClick={(e) => e.stopPropagation()} />
        <Tooltip text={t("agents.copyMemberTip")}>
          <button type="button" className="m3-tonal-btn sm" onClick={(e) => { e.stopPropagation(); onCopyJson(); }}>📋</button>
        </Tooltip>
        <Tooltip text={t("agents.duplicateMemberTip")}>
          <button type="button" className="m3-tonal-btn sm" onClick={(e) => { e.stopPropagation(); onDuplicate(); }}>⧉</button>
        </Tooltip>
        <Tooltip text={t("agents.guide.title")}>
          <button type="button" className="m3-tonal-btn sm" onClick={(e) => { e.stopPropagation(); onOpenHelp(); }}>?</button>
        </Tooltip>
        <button type="button" className="m3-outlined-btn" style={{ padding: "4px 10px", fontSize: 11 }} onClick={(e) => { e.stopPropagation(); onRemove(); }}>✕</button>
      </div>
      {open && (
        <div style={{ marginTop: 12, display: "flex", flexDirection: "column", gap: 12 }}>
          <ModelSelect
            label={t("agents.model")}
            value={member.modelId}
            onChange={(modelId) => onChange({ modelId })}
          />

          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10 }}>
            <div>
              <label className="form-label">{t("agents.role")}</label>
              <select className="m3-input" value={member.role} onChange={(e) => {
                const roleId = e.target.value;
                onChange(applyRoleDefaults(member, roleId, lang));
              }}>
                {AGENT_ROLES.map((r) => <option key={r.id} value={r.id}>{r[lang]}</option>)}
              </select>
            </div>
            <div>
              <label className="form-label">{t("agents.trigger")}</label>
              <select className="m3-input" value={member.trigger || "always"} onChange={(e) => onChange({ trigger: e.target.value } as Partial<AgentMember>)}>
                {TRIGGER_CONDITIONS.map((tr) => <option key={tr.id} value={tr.id}>{tr[lang]}</option>)}
              </select>
            </div>
          </div>

          <div>
            <label className="form-label">{t("agents.systemPrompt")}</label>
            <textarea
              className="m3-input"
              rows={3}
              value={member.systemPrompt || ""}
              onChange={(e) => onChange({ systemPrompt: e.target.value, systemPromptCustomized: true } as Partial<AgentMember>)}
              placeholder={t("agents.systemPromptPh")}
            />
            {!member.systemPromptCustomized && (
              <p className="field-hint">{t("agents.roleAutoPrompt")}</p>
            )}
          </div>

          <div>
            <label className="form-label">{t("agents.tools")}</label>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
              {AGENT_TOOLS.map((tool) => {
                const active = (member.tools || []).includes(tool.id);
                return (
                  <button key={tool.id} type="button" className={`m3-chip ${active ? "active" : ""}`}
                    onClick={() => {
                      const tools = member.tools || [];
                      onChange({ tools: active ? tools.filter((x) => x !== tool.id) : [...tools, tool.id] } as Partial<AgentMember>);
                    }}>
                    {tool[lang]}
                  </button>
                );
              })}
            </div>
          </div>

          <div>
            <label className="form-label">{t("agents.permissions")}</label>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
              {(["internet", "camera", "microphone", "screen", "files", "tools", "canDelegate", "veto", "stm", "ltm", "sharedMemory"] as const).map((k) => {
                const perms = member.permissions as Record<string, boolean>;
                const key = k === "canDelegate" ? "canDelegate" : k === "sharedMemory" ? "sharedMemory" : k;
                if (!(key in perms) && key !== "sharedMemory" && key !== "files" && key !== "tools" && key !== "veto") return null;
                const val = key === "sharedMemory" ? (member.permissions as { sharedMemory?: boolean }).sharedMemory : perms[key];
                return (
                  <button key={k} type="button" className={`m3-chip ${val ? "active" : ""}`}
                    onClick={() => onChange({
                      permissions: { ...member.permissions, [key]: !val },
                    })}>
                    {t(`agents.perm.${k}`)}
                  </button>
                );
              })}
            </div>
          </div>

          <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 8 }}>
            <div>
              <label className="form-label">RAM (MB)</label>
              <input type="number" className="m3-input" value={res.ramLimitMb as number} onChange={(e) => onChange({ resources: { ...res, ramLimitMb: Number(e.target.value) } } as Partial<AgentMember>)} />
            </div>
            <div>
              <label className="form-label">{t("agents.maxTokens")}</label>
              <input type="number" className="m3-input" value={res.maxTokens as number} onChange={(e) => onChange({ resources: { ...res, maxTokens: Number(e.target.value) } } as Partial<AgentMember>)} />
            </div>
            <div>
              <label className="form-label">Temperature</label>
              <input type="number" step={0.1} className="m3-input" value={res.temperature as number} onChange={(e) => onChange({ resources: { ...res, temperature: Number(e.target.value) } } as Partial<AgentMember>)} />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
