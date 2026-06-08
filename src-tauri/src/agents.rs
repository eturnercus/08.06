use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::memory::MemoryStore;
use crate::network::{FetchParams, NetworkManager};
use crate::settings::AppSettings;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTask {
    pub id: String,
    pub group_id: String,
    pub prompt: String,
    pub status: String,
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
    ) -> Result<AgentTask, String> {
        let group = settings
            .agent_groups
            .iter()
            .find(|g| g.id == group_id && g.enabled)
            .ok_or("Группа агентов не найдена или отключена")?;

        let task_id = Uuid::new_v4().to_string();
        let mut task = AgentTask {
            id: task_id.clone(),
            group_id: group_id.to_string(),
            prompt: prompt.to_string(),
            status: "running".into(),
            rounds: Vec::new(),
            created_at: Utc::now(),
        };

        let max_rounds = group.max_rounds.max(1);
        let mut context = prompt.to_string();

        for round in 0..max_rounds {
            let mut round_msgs = Vec::new();
            let members: Vec<_> = if group.parallel_execution {
                group.members.clone()
            } else {
                group.members.clone()
            };

            for member in &members {
                let injection = build_agent_injection(settings, &member.id, &context);
                let response = simulate_agent_response(&member, &injection, settings);

                let mut used_internet = false;
                if member.permissions.internet && settings.network.allow_internet {
                    let fetch_result = network
                        .fetch(FetchParams {
                            url: "https://huggingface.co/api/models?limit=1".into(),
                            method: "GET".into(),
                            body: None,
                            agent_id: Some(member.id.clone()),
                            chat_id: None,
                            allow_internet: true,
                            isolation_mode: settings.network.isolation_mode.clone(),
                            api_endpoints: settings.network.api_only_endpoints.clone(),
                        })
                        .await;
                    used_internet = fetch_result.is_ok();
                }

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
                        0.6,
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
                    timestamp: Utc::now(),
                });
            }

            context = round_msgs
                .iter()
                .map(|m| format!("[{}]: {}", m.agent_name, m.content))
                .collect::<Vec<_>>()
                .join("\n");

            task.rounds.push(AgentRound {
                round_number: round + 1,
                messages: round_msgs,
            });
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

fn build_agent_injection(settings: &AppSettings, agent_id: &str, user_msg: &str) -> String {
    let inj = &settings.global_message_injection;
    if !inj.enabled {
        return user_msg.to_string();
    }
    let mut parts = Vec::new();
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
    parts.push(user_msg.to_string());
    if !inj.user_suffix.is_empty() {
        parts.push(inj.user_suffix.clone());
    }
    parts.join("\n")
}

fn simulate_agent_response(
    member: &crate::settings::AgentMember,
    injection: &str,
    settings: &AppSettings,
) -> String {
    format!(
        "[{} / {}] Обработано {} символов. RAM лимит: {}MB, ядра: {:?}. Ответ на основе локальной модели '{}'.",
        member.name,
        member.role,
        injection.len(),
        settings.system.ram_limit_mb,
        settings.system.cpu_cores,
        member.model_id
    )
}
