use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::inference::InferenceEngine;
use crate::memory::MemoryStore;
use crate::network::NetworkManager;
use crate::settings::{AgentMember, AppSettings};
use crate::settings_engine::{self, check_user_input, filter_model_output, swarm_agent_count};
use crate::stream_sink::{AgentStreamSink, stream_buffer_ms};
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
}

impl AgentOrchestrator {
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
        }
    }

    pub async fn run_team_task(
        &self,
        settings: &AppSettings,
        group_id: &str,
        prompt: &str,
        memory: &MemoryStore,
        network: &NetworkManager,
        inference: &InferenceEngine,
        app: AppHandle,
    ) -> Result<AgentTask, String> {
        let prompt = check_user_input(settings, prompt)?;
        settings_engine::audit_log(settings, "agents", &format!("team task group={group_id}"));
        let group = settings
            .agent_groups
            .iter()
            .find(|g| g.id == group_id && g.enabled)
            .ok_or("Группа агентов не найдена или отключена")?;

        let task_id = Uuid::new_v4().to_string();
        let mode = group.orchestration_mode.clone();
        let mut task = AgentTask {
            id: task_id.clone(),
            group_id: group_id.to_string(),
            prompt: prompt.to_string(),
            status: "running".into(),
            orchestration_mode: mode.clone(),
            rounds: Vec::new(),
            created_at: Utc::now(),
        };

        let max_rounds = group.max_rounds.max(1).min(50);
        let mut context = prompt.to_string();
        let mut had_failure = false;

        for round in 0..max_rounds {
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

            let mut round_msgs = Vec::new();

            for member in &members_to_run {
                let injection = build_agent_injection(settings, &member.id, &context, member);
                let (response, used_internet, tools_used) = execute_member(
                    member,
                    &injection,
                    settings,
                    network,
                    &prompt,
                    &mode,
                    inference,
                    memory,
                    &task_id,
                    &app,
                )
                .await;

                if member.permissions.stm {
                    memory.add_stm(
                        &task_id,
                        &member.role,
                        &response,
                        settings.memory.stm_max_tokens,
                    );
                }
                if member.permissions.ltm {
                    memory.add_ltm(
                        &task_id,
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

        task.status = "completed".into();
        self.tasks.write().insert(task_id, task.clone());
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
    prompt: &str,
    mode: &str,
    inference: &InferenceEngine,
    memory: &MemoryStore,
    task_id: &str,
    app: &AppHandle,
) -> (String, bool, Vec<String>) {
    let tools_used: Vec<String> = member.tools.clone();
    let mut used_internet = false;
    let mut tool_notes = Vec::new();

    if member.permissions.internet
        && settings.network.allow_internet
        && member.tools.iter().any(|t| t == "web_search")
    {
        let q = prompt.chars().take(120).collect::<String>();
        if let Ok(log) = network.web_search(&q, Some(member.id.clone())).await {
            used_internet = !log.blocked;
            tool_notes.push(format!(
                "web_search: {}",
                log.response_preview.chars().take(80).collect::<String>()
            ));
        }
    }

    if member.tools.iter().any(|t| t == "memory_query") {
        let recalled = settings_engine::recall_ltm(memory, settings, task_id, prompt, None);
        if !recalled.is_empty() {
            tool_notes.push(format!("memory_query: {} записей LTM", recalled.len()));
        }
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

    let response = match inference.generate_reply(
        settings,
        &model_id,
        &member.system_prompt,
        &full_prompt,
        &mut agent_sink,
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

    (response, used_internet, tools_used)
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
