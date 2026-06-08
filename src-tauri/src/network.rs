use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkRequestLog {
    pub id: String,
    pub agent_id: Option<String>,
    pub chat_id: Option<String>,
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub request_headers: HashMapStr,
    pub response_preview: String,
    pub duration_ms: u64,
    pub blocked: bool,
    pub block_reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}

type HashMapStr = std::collections::HashMap<String, String>;

pub struct NetworkManager {
    logs: RwLock<VecDeque<NetworkRequestLog>>,
    client: Client,
    max_logs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchParams {
    pub url: String,
    pub method: String,
    pub body: Option<String>,
    pub agent_id: Option<String>,
    pub chat_id: Option<String>,
    pub allow_internet: bool,
    pub isolation_mode: String,
    pub api_endpoints: Vec<String>,
}

impl NetworkManager {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("NeuroForge/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            logs: RwLock::new(VecDeque::new()),
            client,
            max_logs: 500,
        }
    }

    pub fn get_logs(&self) -> Vec<NetworkRequestLog> {
        self.logs.read().iter().cloned().collect()
    }

    fn push_log(&self, log: NetworkRequestLog) {
        let mut logs = self.logs.write();
        logs.push_front(log);
        while logs.len() > self.max_logs {
            logs.pop_back();
        }
    }

    fn is_allowed(&self, url: &str, params: &FetchParams) -> Result<(), String> {
        if !params.allow_internet {
            return Err("Интернет отключён для этого чата/агента".into());
        }
        match params.isolation_mode.as_str() {
            "full" => Err("Полная изоляция: сетевые запросы запрещены".into()),
            "api_only" => {
                let allowed = params.api_endpoints.iter().any(|ep| {
                    if ep.contains('*') {
                        let prefix = ep.split('*').next().unwrap_or("");
                        url.starts_with(prefix)
                    } else {
                        url.starts_with(ep)
                    }
                });
                if allowed {
                    Ok(())
                } else {
                    Err(format!("URL не в белом списке API: {url}"))
                }
            }
            "none" | _ => Ok(()),
        }
    }

    pub async fn web_search(&self, query: &str, agent_id: Option<String>) -> Result<NetworkRequestLog, String> {
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1",
            urlencoding::encode(query)
        );
        self.fetch(FetchParams {
            url,
            method: "GET".into(),
            body: None,
            agent_id,
            chat_id: None,
            allow_internet: true,
            isolation_mode: "api_only".into(),
            api_endpoints: vec![
                "https://api.duckduckgo.com".into(),
                "https://*.duckduckgo.com".into(),
            ],
        })
        .await
    }

    pub async fn fetch(&self, params: FetchParams) -> Result<NetworkRequestLog, String> {
        let id = Uuid::new_v4().to_string();
        let start = std::time::Instant::now();

        if let Err(reason) = self.is_allowed(&params.url, &params) {
            let log = NetworkRequestLog {
                id: id.clone(),
                agent_id: params.agent_id.clone(),
                chat_id: params.chat_id.clone(),
                method: params.method.clone(),
                url: params.url.clone(),
                status: None,
                request_headers: HashMapStr::new(),
                response_preview: String::new(),
                duration_ms: start.elapsed().as_millis() as u64,
                blocked: true,
                block_reason: Some(reason.clone()),
                timestamp: Utc::now(),
            };
            self.push_log(log.clone());
            return Err(reason);
        }

        let method = params.method.to_uppercase();
        let mut req = match method.as_str() {
            "GET" => self.client.get(&params.url),
            "POST" => self.client.post(&params.url),
            "PUT" => self.client.put(&params.url),
            "DELETE" => self.client.delete(&params.url),
            _ => self.client.get(&params.url),
        };
        if let Some(body) = &params.body {
            req = req.body(body.clone());
        }

        let result = req.send().await;
        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let preview = resp
                    .text()
                    .await
                    .unwrap_or_default()
                    .chars()
                    .take(500)
                    .collect();
                let log = NetworkRequestLog {
                    id,
                    agent_id: params.agent_id,
                    chat_id: params.chat_id,
                    method: params.method,
                    url: params.url,
                    status: Some(status),
                    request_headers: HashMapStr::new(),
                    response_preview: preview,
                    duration_ms,
                    blocked: false,
                    block_reason: None,
                    timestamp: Utc::now(),
                };
                self.push_log(log.clone());
                Ok(log)
            }
            Err(e) => {
                let log = NetworkRequestLog {
                    id,
                    agent_id: params.agent_id,
                    chat_id: params.chat_id,
                    method: params.method,
                    url: params.url,
                    status: None,
                    request_headers: HashMapStr::new(),
                    response_preview: e.to_string(),
                    duration_ms,
                    blocked: false,
                    block_reason: Some(e.to_string()),
                    timestamp: Utc::now(),
                };
                self.push_log(log.clone());
                Err(e.to_string())
            }
        }
    }
}
