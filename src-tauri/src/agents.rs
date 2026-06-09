use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::agent_webview::AgentWebView;
use crate::agent_workspace;
use crate::desktop_agent::{self, DesktopAgent};
use crate::inference::InferenceEngine;
use crate::inference_cancel::CancelRegistry;
use crate::memory::MemoryStore;
use crate::network::NetworkManager;
use crate::settings::{AgentGroupConfig, AgentMember, AppSettings};
use crate::settings_engine::{self, check_user_input, filter_model_output, swarm_agent_count};
use crate::stream_sink::{
    emit_orchestration, AgentOrchestrationPayload, AgentStreamSink, stream_buffer_ms,
};
use std::sync::atomic::Ordering;
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTask {
    pub id: String,
    pub group_id: String,
    pub prompt: String,
    pub status: String,
    pub orchestration_mode: String,
    pub rounds: Vec<AgentRound>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub final_response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRound {
    pub round_number: u32,
    pub messages: Vec<AgentMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMessage {
    pub agent_id: String,
    pub agent_name: String,
    pub role: String,
    pub content: String,
    pub used_internet: bool,
    pub tools_used: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

pub struct AgentOrchestrator {
    tasks: RwLock<HashMap<String, AgentTask>>,
    current_task_id: RwLock<Option<String>>,
}

impl AgentOrchestrator {
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
            current_task_id: RwLock::new(None),
        }
    }

    pub fn current_task_id(&self) -> Option<String> {
        self.current_task_id.read().clone()
    }

    pub fn stop_task(&self, cancel_reg: &CancelRegistry, task_id: &str) {
        cancel_reg.cancel(task_id);
    }

    pub fn stop_agent(&self, cancel_reg: &CancelRegistry, task_id: &str, agent_id: &str) {
        cancel_reg.cancel(&format!("{task_id}:{agent_id}"));
    }

    pub async fn run_team_task(
        &self,
        settings: &AppSettings,
        group_id: &str,
        prompt: &str,
        memory: &MemoryStore,
        network: &NetworkManager,
        inference: &InferenceEngine,
        desktop: &DesktopAgent,
        webview: &AgentWebView,
        cancel_reg: &CancelRegistry,
        app: AppHandle,
        chat_id: Option<String>,
    ) -> Result<AgentTask, String> {
        let prompt = check_user_input(settings, prompt)?;
        settings_engine::audit_log(settings, "agents", &format!("team task group={group_id}"));
        let group = settings
            .agent_groups
            .iter()
            .find(|g| g.id == group_id && g.enabled)
            .ok_or("Группа агентов не найдена или отключена")?;

        let task_id = Uuid::new_v4().to_string();
        *self.current_task_id.write() = Some(task_id.clone());
        let _task_cancel = cancel_reg.begin(&task_id);
        let memory_scope = chat_id.as_deref().unwrap_or(&task_id);
        let workspace_path = chat_id
            .as_deref()
            .and_then(|id| settings.per_chat_overrides.get(id))
            .and_then(|o| o.workspace_path.clone())
            .filter(|p| !p.trim().is_empty());
        let mode = group.orchestration_mode.clone();
        let mut task = AgentTask {
            id: task_id.clone(),
            group_id: group_id.to_string(),
            prompt: prompt.to_string(),
            status: "running".into(),
            orchestration_mode: mode.clone(),
            rounds: Vec::new(),
            created_at: Utc::now(),
            final_response: String::new(),
        };

        emit_orchestration(
            &app,
            AgentOrchestrationPayload {
                task_id: task_id.clone(),
                group_id: group.id.clone(),
                group_name: group.name.clone(),
                orchestration_mode: mode.clone(),
                round: 0,
                phase: "task_start".into(),
                agent_id: None,
                agent_name: None,
                model_id: None,
                status: "running".into(),
                message: Some(prompt.chars().take(120).collect()),
            },
        );

        let max_rounds = group.max_rounds.max(1).min(50);
        let mut context = prompt.to_string();
        let mut had_failure = false;

        for round in 0..max_rounds {
            if cancel_reg.is_cancelled(&task_id) {
                task.status = "cancelled".into();
                break;
            }
            let ordered = order_members(&group.members, &mode, round);
            let active: Vec<_> = ordered
                .into_iter()
                .filter(|m| should_run_member(m, &prompt, round, had_failure))
                .collect();

            if active.is_empty() {
                break;
            }

            let parallel = group.parallel_execution
                || matches!(mode.as_str(), "parallel" | "map_reduce" | "expert_panel");

            let swarm_n = swarm_agent_count(settings, active.len());
            let members_to_run: Vec<_> = if settings.innovation.swarm_intelligence && swarm_n < active.len() {
                active.iter().take(swarm_n).cloned().collect()
            } else if parallel {
                active
            } else {
                match mode.as_str() {
                    "round_robin" => vec![active[round as usize % active.len()].clone()],
                    "smart_router" => {
                        let idx = (prompt.len() + round as usize) % active.len();
                        vec![active[idx].clone()]
                    }
                    "voting" | "debate" if round % 2 == 1 => active.iter().take(2).cloned().collect(),
                    _ => active,
                }
            };

            emit_orchestration(
                &app,
                AgentOrchestrationPayload {
                    task_id: task_id.clone(),
                    group_id: group.id.clone(),
                    group_name: group.name.clone(),
                    orchestration_mode: mode.clone(),
                    round: round + 1,
                    phase: "round_start".into(),
                    agent_id: None,
                    agent_name: None,
                    model_id: None,
                    status: "running".into(),
                    message: Some(format!("{} agents", members_to_run.len())),
                },
            );

            let mut round_msgs = Vec::new();

            for member in &members_to_run {
                if cancel_reg.is_cancelled(&task_id)
                    || cancel_reg.is_cancelled(&format!("{}:{}", task_id, member.id))
                {
                    round_msgs.push(AgentMessage {
                        agent_id: member.id.clone(),
                        agent_name: member.name.clone(),
                        role: member.role.clone(),
                        content: "[остановлено]".into(),
                        used_internet: false,
                        tools_used: vec![],
                        timestamp: Utc::now(),
                    });
                    continue;
                }

                emit_orchestration(
                    &app,
                    AgentOrchestrationPayload {
                        task_id: task_id.clone(),
                        group_id: group.id.clone(),
                        group_name: group.name.clone(),
                        orchestration_mode: mode.clone(),
                        round: round + 1,
                        phase: "agent_start".into(),
                        agent_id: Some(member.id.clone()),
                        agent_name: Some(member.name.clone()),
                        model_id: Some(member.model_id.clone()),
                        status: "running".into(),
                        message: Some(member.role.clone()),
                    },
                );

                let injection = build_agent_injection(settings, &member.id, &context, member);
                let (response, used_internet, tools_used) = execute_member(
                    member,
                    &injection,
                    settings,
                    network,
                    desktop,
                    webview,
                    &prompt,
                    &mode,
                    inference,
                    memory,
                    &task_id,
                    chat_id.as_deref(),
                    workspace_path.as_deref(),
                    cancel_reg,
                    &app,
                )
                .await;

                emit_orchestration(
                    &app,
                    AgentOrchestrationPayload {
                        task_id: task_id.clone(),
                        group_id: group.id.clone(),
                        group_name: group.name.clone(),
                        orchestration_mode: mode.clone(),
                        round: round + 1,
                        phase: "agent_done".into(),
                        agent_id: Some(member.id.clone()),
                        agent_name: Some(member.name.clone()),
                        model_id: Some(member.model_id.clone()),
                        status: if response.starts_with('[') && response.contains("остановлено") {
                            "cancelled".into()
                        } else {
                            "ok".into()
                        },
                        message: Some(response.chars().take(160).collect()),
                    },
                );

                if member.permissions.stm {
                    memory.add_stm(
                        memory_scope,
                        &member.role,
                        &response,
                        settings.memory.stm_max_tokens,
                    );
                }
                if member.permissions.ltm {
                    memory.add_ltm(
                        memory_scope,
                        &member.model_id,
                        &response,
                        importance_for_role(&member.role),
                        vec![member.role.clone()],
                        group.shared_memory,
                        Some(member.id.clone()),
                    );
                }

                round_msgs.push(AgentMessage {
                    agent_id: member.id.clone(),
                    agent_name: member.name.clone(),
                    role: member.role.clone(),
                    content: response,
                    used_internet,
                    tools_used,
                    timestamp: Utc::now(),
                });
            }

            if round_msgs.is_empty() {
                had_failure = true;
                continue;
            }

            context = synthesize_context(&mode, &round_msgs, &context, group.consensus_threshold);

            task.rounds.push(AgentRound {
                round_number: round + 1,
                messages: round_msgs,
            });

            if mode == "chain_of_thought" && round + 1 >= 2 {
                break;
            }
        }

        if task.status != "cancelled" {
            task.final_response = finalize_team_response(
                group,
                settings,
                inference,
                &task,
                &context,
                cancel_reg,
                &task_id,
                &app,
            )
            .await;
            task.status = "completed".into();
        }

        emit_orchestration(
            &app,
            AgentOrchestrationPayload {
                task_id: task_id.clone(),
                group_id: group.id.clone(),
                group_name: group.name.clone(),
                orchestration_mode: mode.clone(),
                round: task.rounds.len() as u32,
                phase: "task_done".into(),
                agent_id: None,
                agent_name: None,
                model_id: None,
                status: task.status.clone(),
                message: Some(task.final_response.chars().take(200).collect()),
            },
        );

        self.tasks.write().insert(task_id.clone(), task.clone());
        *self.current_task_id.write() = None;
        cancel_reg.finish(&task_id);
        Ok(task)
    }

    pub fn get_task(&self, id: &str) -> Option<AgentTask> {
        self.tasks.read().get(id).cloned()
    }

    pub fn list_tasks(&self) -> Vec<AgentTask> {
        self.tasks.read().values().cloned().collect()
    }
}

fn order_members(members: &[AgentMember], mode: &str, round: u32) -> Vec<AgentMember> {
    let mut sorted = members.to_vec();
    match mode {
        "hierarchical" => {
            sorted.sort_by(|a, b| {
                let rank = |r: &str| match r {
                    "leader" | "custom_manager" => 0,
                    "router" | "analyst" => 1,
                    _ => 2,
                };
                rank(&a.role).cmp(&rank(&b.role))
            });
        }
        "expert_panel" => {
            sorted.sort_by_key(|m| m.resources.execution_order);
        }
        "round_robin" => {
            if !sorted.is_empty() {
                let shift = (round as usize) % sorted.len();
                sorted.rotate_left(shift);
            }
        }
        _ => {
            sorted.sort_by_key(|m| m.resources.execution_order);
        }
    }
    sorted
}

fn should_run_member(member: &AgentMember, prompt: &str, round: u32, had_failure: bool) -> bool {
    match member.trigger.as_str() {
        "keyword" => {
            !member.trigger_keyword.is_empty()
                && prompt
                    .to_lowercase()
                    .contains(&member.trigger_keyword.to_lowercase())
        }
        "delegation" => member.permissions.can_delegate,
        "round" => round > 0,
        "failure" => had_failure,
        "custom" => !member.system_prompt.is_empty(),
        _ => true,
    }
}

async fn execute_member(
    member: &AgentMember,
    injection: &str,
    settings: &AppSettings,
    network: &NetworkManager,
    desktop: &DesktopAgent,
    webview: &AgentWebView,
    prompt: &str,
    mode: &str,
    inference: &InferenceEngine,
    memory: &MemoryStore,
    task_id: &str,
    chat_id: Option<&str>,
    workspace_path: Option<&str>,
    cancel_reg: &CancelRegistry,
    app: &AppHandle,
) -> (String, bool, Vec<String>) {
    let agent_cancel_key = format!("{task_id}:{}", member.id);
    if cancel_reg.is_cancelled(task_id) || cancel_reg.is_cancelled(&agent_cancel_key) {
        return ("[остановлено]".into(), false, vec![]);
    }
    let agent_cancel = cancel_reg.begin(&agent_cancel_key);
    let tools_used: Vec<String> = member.tools.clone();
    let mut used_internet = false;
    let mut tool_notes = Vec::new();

    if member.permissions.internet
        && settings.network.allow_internet
        && member.tools.iter().any(|t| t == "web_search")
    {
        let q = prompt.chars().take(120).collect::<String>();
        match network
            .web_search(
                &q,
                Some(member.id.clone()),
                Some(task_id.to_string()),
                true,
                settings,
            )
            .await
        {
            Ok(log) => {
                used_internet = !log.blocked;
                let status = if log.blocked { "blocked" } else { "ok" };
                tool_notes.push(format!(
                    "web_search [{status}]: {}",
                    log.response_preview.chars().take(120).collect::<String>()
                ));
            }
            Err(e) => {
                tool_notes.push(format!("web_search [error]: {e}"));
            }
        }
    } else if member.tools.iter().any(|t| t == "web_search") && !member.permissions.internet {
        tool_notes.push("web_search [skipped]: интернет отключён для этого агента".into());
    }

    let browser_tools = member.tools.iter().any(|t| {
        t == "browser_navigate" || t == "browser_click" || t == "browser_search"
    });
    if browser_tools {
        if !member.permissions.screen {
            tool_notes.push("browser_* [skipped]: нет разрешения «Экран»".into());
        } else if !settings.devices.browser_automation_enabled || !settings.devices.desktop_control_enabled {
            tool_notes.push("browser_* [skipped]: включите Browser automation и Desktop control".into());
        } else if !member.permissions.internet {
            tool_notes.push("browser_* [skipped]: интернет отключён для агента".into());
        } else {
            desktop.set_dual_mouse(true, app);
            used_internet = true;
            if member.tools.iter().any(|t| t == "browser_navigate") {
                if let Some(url) = desktop_agent::extract_url_from_text(prompt) {
                    match desktop
                        .navigate(
                            app,
                            network,
                            settings,
                            &url,
                            chat_id.map(str::to_string),
                            Some(member.id.clone()),
                            webview,
                        )
                        .await
                    {
                        Ok(msg) => tool_notes.push(format!("browser_navigate [ok]: {msg}")),
                        Err(e) => tool_notes.push(format!("browser_navigate [error]: {e}")),
                    }
                }
            }
            if member.tools.iter().any(|t| t == "browser_search") {
                let q = prompt.chars().take(120).collect::<String>();
                match desktop
                    .search(
                        app,
                        network,
                        settings,
                        &q,
                        chat_id.map(str::to_string),
                        Some(member.id.clone()),
                        webview,
                    )
                    .await
                {
                    Ok(msg) => tool_notes.push(format!("browser_search [ok]: {msg}")),
                    Err(e) => tool_notes.push(format!("browser_search [error]: {e}")),
                }
            }
            if member.tools.iter().any(|t| t == "browser_click") {
                let idx = find_link_index_in_prompt(prompt, &desktop.snapshot(webview).browser.links);
                match desktop
                    .click_link(
                        app,
                        network,
                        settings,
                        idx,
                        chat_id.map(str::to_string),
                        Some(member.id.clone()),
                        webview,
                    )
                    .await
                {
                    Ok(msg) => tool_notes.push(format!("browser_click [ok]: {msg}")),
                    Err(e) => tool_notes.push(format!("browser_click [error]: {e}")),
                }
            }
        }
    }

    if member.tools.iter().any(|t| t == "memory_query") {
        let recalled = settings_engine::recall_ltm(memory, settings, task_id, prompt);
        if !recalled.is_empty() {
            tool_notes.push(format!("memory_query: {} записей LTM", recalled.len()));
        }
    }

    if member.permissions.files {
        if member.tools.iter().any(|t| t == "file_read") {
            if let Some(ws) = workspace_path {
                if let Some(path) = agent_workspace::extract_file_path_from_prompt(prompt) {
                    match agent_workspace::read_file(ws, &path, 512_000) {
                        Ok(text) => {
                            tool_notes.push(format!("file_read [{path}]: {} симв.", text.chars().count()));
                        }
                        Err(e) => tool_notes.push(format!("file_read [error]: {e}")),
                    }
                }
            } else if member.tools.iter().any(|t| t == "file_read") {
                tool_notes.push("file_read [skipped]: задайте рабочую папку в свойствах чата".into());
            }
        }
        if member.tools.iter().any(|t| t == "file_write") {
            if let Some(ws) = workspace_path {
                let out = format!("agent-{task_id}-{}.md", member.id);
                let body = format!("# {}\n\n{}\n", member.name, prompt.chars().take(2000).collect::<String>());
                match agent_workspace::write_file(ws, &out, &body) {
                    Ok(msg) => tool_notes.push(format!("file_write [ok]: {msg}")),
                    Err(e) => tool_notes.push(format!("file_write [error]: {e}")),
                }
            }
        }
    }

    if member.tools.iter().any(|t| t == "image_analyze") {
        tool_notes.push("image_analyze: назначьте vision-модель агенту (отдельный участник с GGUF multimodal)".into());
    }

    let model_id = if member.model_id.is_empty() {
        "default".to_string()
    } else {
        member.model_id.clone()
    };

    let mut full_prompt = injection.to_string();
    if !tool_notes.is_empty() {
        full_prompt.push_str("\n[tools]\n");
        full_prompt.push_str(&tool_notes.join("\n"));
    }

    let stream_on =
        settings.inference.streaming || settings.innovation.thought_streaming;
    let mut agent_sink = if stream_on {
        Some(AgentStreamSink::new(
            app.clone(),
            task_id.to_string(),
            member.id.clone(),
            member.name.clone(),
            stream_buffer_ms(settings),
        ))
    } else {
        None
    };

    if cancel_reg.is_cancelled(task_id) || agent_cancel.load(Ordering::SeqCst) {
        cancel_reg.finish(&agent_cancel_key);
        return ("[остановлено]".into(), used_internet, tools_used);
    }

    let max_tokens = member.resources.max_tokens.max(64);
    let temperature = member.resources.temperature.clamp(0.0, 2.0);

    let response = match inference.generate_reply(
        settings,
        &model_id,
        &member.system_prompt,
        &full_prompt,
        &mut agent_sink,
        max_tokens,
        temperature,
        Some(&agent_cancel),
    ) {
        Ok(text) => {
            if let Some(ref mut s) = agent_sink {
                s.finish();
            }
            filter_model_output(settings, &text)
        }
        Err(_) => format!(
            "[{} / {}] Режим: {}. Инструменты: {}. (модель {} недоступна — укажите GGUF в настройках агента){}",
            member.name,
            member.role,
            mode,
            if tools_used.is_empty() {
                "—".into()
            } else {
                tools_used.join(", ")
            },
            model_id,
            if tool_notes.is_empty() {
                String::new()
            } else {
                format!(" Заметки: {}", tool_notes.join("; "))
            }
        ),
    };

    cancel_reg.finish(&agent_cancel_key);
    (response, used_internet, tools_used)
}

async fn finalize_team_response(
    group: &AgentGroupConfig,
    settings: &AppSettings,
    inference: &InferenceEngine,
    task: &AgentTask,
    context: &str,
    cancel_reg: &CancelRegistry,
    task_id: &str,
    app: &AppHandle,
) -> String {
    let last_round = match task.rounds.last() {
        Some(r) => r,
        None => return context.to_string(),
    };

    let conflict = if group.conflict_mode.is_empty() {
        "consensus"
    } else {
        group.conflict_mode.as_str()
    };

    let joined = last_round
        .messages
        .iter()
        .map(|m| format!("[{} / {}]:\n{}", m.agent_name, m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n\n");

    let synthesizer = group
        .members
        .iter()
        .find(|m| m.role == "summarizer")
        .or_else(|| group.members.iter().find(|m| m.role == "leader"))
        .or_else(|| group.members.iter().find(|m| m.role == "custom_manager"));

    if let Some(member) = synthesizer {
        if !cancel_reg.is_cancelled(task_id) {
            let synth_prompt = format!(
                "Исходная задача пользователя:\n{}\n\nРабота команды (последний раунд):\n{}\n\nСформируй ОДИН финальный ответ пользователю. Без перечисления ролей и без дублирования.",
                task.prompt, joined
            );
            let agent_cancel_key = format!("{task_id}:synth:{}", member.id);
            let flag = cancel_reg.begin(&agent_cancel_key);
            if let Ok(text) = inference.generate_reply(
                settings,
                &member.model_id,
                "Ты составляешь единый итоговый ответ команды агентов.",
                &synth_prompt,
                &mut None,
                member.resources.max_tokens.max(512),
                member.resources.temperature.clamp(0.0, 2.0),
                Some(&flag),
            ) {
                cancel_reg.finish(&agent_cancel_key);
                let filtered = filter_model_output(settings, &text);
                if !filtered.trim().is_empty() {
                    emit_orchestration(
                        app,
                        AgentOrchestrationPayload {
                            task_id: task_id.to_string(),
                            group_id: group.id.clone(),
                            group_name: group.name.clone(),
                            orchestration_mode: task.orchestration_mode.clone(),
                            round: task.rounds.len() as u32,
                            phase: "synthesis".into(),
                            agent_id: Some(member.id.clone()),
                            agent_name: Some(member.name.clone()),
                            model_id: Some(member.model_id.clone()),
                            status: "ok".into(),
                            message: Some("final_response".into()),
                        },
                    );
                    return filtered;
                }
            }
            cancel_reg.finish(&agent_cancel_key);
        }
    }

    merge_last_round(last_round, conflict)
}

fn merge_last_round(round: &AgentRound, conflict_mode: &str) -> String {
    let msgs = &round.messages;
    if msgs.is_empty() {
        return String::new();
    }
    match conflict_mode {
        "leader_decides" => msgs
            .iter()
            .find(|m| m.role == "leader" || m.role == "custom_manager")
            .or_else(|| msgs.first())
            .map(|m| m.content.clone())
            .unwrap_or_default(),
        "escalate_user" => {
            let body = msgs
                .iter()
                .map(|m| format!("**{}** ({}):\n{}", m.agent_name, m.role, m.content))
                .collect::<Vec<_>>()
                .join("\n\n");
            format!("Команда подготовила несколько вариантов — выберите или уточните:\n\n{body}")
        }
        _ => {
            if msgs.len() == 1 {
                msgs[0].content.clone()
            } else {
                msgs.last().map(|m| m.content.clone()).unwrap_or_default()
            }
        }
    }
}

fn synthesize_context(mode: &str, msgs: &[AgentMessage], prev: &str, consensus: f32) -> String {
    let joined = msgs
        .iter()
        .map(|m| format!("[{} / {}]: {}", m.agent_name, m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    match mode {
        "voting" | "debate" => format!("{prev}\n--- round votes (threshold {consensus:.0}) ---\n{joined}"),
        "map_reduce" => format!("{prev}\n--- map phase ---\n{joined}\n--- reduce pending ---"),
        "chain_of_thought" => format!("{prev}\n--- reasoning chain ---\n{joined}"),
        _ => joined,
    }
}

fn find_link_index_in_prompt(prompt: &str, links: &[desktop_agent::BrowserLink]) -> usize {
    let lower = prompt.to_lowercase();
    for link in links {
        if !link.text.is_empty() && lower.contains(&link.text.to_lowercase()) {
            return link.index;
        }
    }
    links.first().map(|l| l.index).unwrap_or(0)
}

fn importance_for_role(role: &str) -> f32 {
    match role {
        "leader" | "fact_checker" | "analyst" => 0.85,
        "researcher" | "programmer" => 0.75,
        "summarizer" | "reviewer" => 0.65,
        _ => 0.55,
    }
}

fn build_agent_injection(
    settings: &AppSettings,
    agent_id: &str,
    user_msg: &str,
    member: &AgentMember,
) -> String {
    let inj = &settings.global_message_injection;
    let mut parts = Vec::new();
    if !member.system_prompt.is_empty() {
        parts.push(format!("[agent] {}", member.system_prompt));
    }
    if inj.enabled {
        if !inj.system_prefix.is_empty() {
            parts.push(inj.system_prefix.clone());
        }
        if let Some(override_ctx) = inj.per_agent_overrides.get(agent_id) {
            parts.push(override_ctx.clone());
        }
        if !inj.hidden_context.is_empty() {
            parts.push(format!("[hidden] {}", inj.hidden_context));
        }
        if inj.inject_timestamp {
            parts.push(format!("[time] {}", Utc::now().to_rfc3339()));
        }
        if inj.inject_locale {
            parts.push(format!("[locale] {}", settings.language));
        }
    }
    parts.push(user_msg.to_string());
    if inj.enabled && !inj.user_suffix.is_empty() {
        parts.push(inj.user_suffix.clone());
    }
    parts.join("\n")
}
