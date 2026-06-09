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
    #[serde(default)]
    pub data_exfiltration_guard: bool,
    #[serde(default)]
    pub audit_enabled: bool,
    #[serde(default)]
    pub block_private_ips: bool,
    #[serde(default)]
    pub network_fingerprint_check: bool,
}

impl NetworkManager {
    fn is_private_url(url: &str) -> bool {
        let lower = url.to_lowercase();
        lower.contains("://127.")
            || lower.contains("://10.")
            || lower.contains("://192.168.")
            || lower.contains("://localhost")
            || lower.contains("://[::1]")
    }

    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Silenium/1.0")
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

    pub fn record_log(&self, log: NetworkRequestLog) {
        self.push_log(log);
    }

    fn push_log(&self, log: NetworkRequestLog) {
        let mut logs = self.logs.write();
        logs.push_front(log);
        while logs.len() > self.max_logs {
            logs.pop_back();
        }
    }

    fn is_allowed(&self, url: &str, params: &FetchParams) -> Result<(), String> {
        if params.block_private_ips && Self::is_private_url(url) {
            return Err("Запрос к приватному IP заблокирован политикой сети".into());
        }
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

    fn fetch_params(
        &self,
        url: String,
        agent_id: Option<String>,
        chat_id: Option<String>,
        allow_internet: bool,
        settings: &crate::settings::AppSettings,
    ) -> FetchParams {
        FetchParams {
            url,
            method: "GET".into(),
            body: None,
            agent_id,
            chat_id,
            allow_internet,
            isolation_mode: settings.network.isolation_mode.clone(),
            api_endpoints: settings.network.api_only_endpoints.clone(),
            data_exfiltration_guard: settings.security.data_exfiltration_guard,
            audit_enabled: settings.security.audit_log_enabled,
            block_private_ips: settings.network.block_private_ips,
            network_fingerprint_check: settings.security.network_fingerprint_check,
        }
    }

    pub async fn web_search(
        &self,
        query: &str,
        agent_id: Option<String>,
        chat_id: Option<String>,
        allow_internet: bool,
        settings: &crate::settings::AppSettings,
    ) -> Result<NetworkRequestLog, String> {
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1",
            urlencoding::encode(query)
        );
        self.fetch(self.fetch_params(
            url,
            agent_id,
            chat_id,
            allow_internet,
            settings,
        ))
        .await
    }

    /// wttr.in — compact weather line (DuckDuckGo Instant Answer has no weather data).
    pub async fn fetch_weather(
        &self,
        location: &str,
        agent_id: Option<String>,
        chat_id: Option<String>,
        allow_internet: bool,
        settings: &crate::settings::AppSettings,
    ) -> Result<NetworkRequestLog, String> {
        let loc = location.trim();
        if loc.is_empty() {
            return Err("Не указан город для погоды".into());
        }
        let url = format!(
            "https://wttr.in/{loc}?format=3&lang=ru",
            loc = urlencoding::encode(loc)
        );
        self.fetch(self.fetch_params(
            url,
            agent_id,
            chat_id,
            allow_internet,
            settings,
        ))
        .await
    }

    /// Полная загрузка HTML для встроенного агент-браузера (без усечения preview).
    pub async fn fetch_page_html(&self, params: FetchParams) -> Result<(u16, String), String> {
        if let Err(reason) = self.is_allowed(&params.url, &params) {
            return Err(reason);
        }
        let method = params.method.to_uppercase();
        let mut req = match method.as_str() {
            "POST" => self.client.post(&params.url),
            "PUT" => self.client.put(&params.url),
            "DELETE" => self.client.delete(&params.url),
            _ => self.client.get(&params.url),
        };
        if let Some(body) = &params.body {
            req = req.body(body.clone());
        }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        let status = resp.status().as_u16();
        let mut html = resp.text().await.unwrap_or_default();
        if html.len() > 2_000_000 {
            html.truncate(2_000_000);
        }
        Ok((status, html))
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
                let mut preview = resp
                    .text()
                    .await
                    .unwrap_or_default()
                    .chars()
                    .take(500)
                    .collect::<String>();
                if params.data_exfiltration_guard {
                    for pat in ["api_key", "password", "secret", "Bearer "] {
                        if preview.to_lowercase().contains(pat) {
                            preview = preview.replace(pat, "[redacted]");
                        }
                    }
                }
                if params.audit_enabled {
                    crate::settings_engine::audit_log_raw(&format!(
                        "network {} {} -> {}",
                        params.method, params.url, preview.chars().take(80).collect::<String>()
                    ));
                }
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

/// Heuristic: user explicitly asks to use the web / weather / lookup.
pub fn message_wants_web_search(message: &str) -> bool {
    let lower = message.to_lowercase();
    const TRIGGERS: &[&str] = &[
        "интернет",
        "в сети",
        "онлайн",
        "посмотри",
        "найди",
        "погугли",
        "загугли",
        "duckduckgo",
        "погод",
        "weather",
        "look up",
        "search the",
        "search for",
        "в интернете",
        "из интернета",
        "узнай в",
    ];
    TRIGGERS.iter().any(|t| lower.contains(t))
}

const QUERY_STOP_WORDS: &[&str] = &[
    "посмотри",
    "посмотреть",
    "найди",
    "найти",
    "узнай",
    "узнать",
    "интернет",
    "интернете",
    "онлайн",
    "погугли",
    "загугли",
    "please",
    "look",
    "online",
    "search",
    "что",
    "там",
    "мне",
    "нужно",
    "надо",
    "хочу",
    "хотел",
    "бы",
    "ли",
];

pub fn is_weather_query(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("погод") || lower.contains("weather")
}

/// City/place after «погода в …» or «weather in …».
pub fn extract_weather_location(message: &str) -> Option<String> {
    if !is_weather_query(message) {
        return None;
    }
    let lower = message.to_lowercase();
    let start = lower
        .find(" в ")
        .or_else(|| lower.find(" in "))
        .map(|i| {
            if lower[i..].starts_with(" в ") {
                i + 3
            } else {
                i + 4
            }
        })?;
    let tail = message[start..].trim();
    let words: Vec<&str> = tail.split_whitespace().collect();
    let mut picked: Vec<&str> = Vec::new();
    for w in words {
        let wl = w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
        if wl.is_empty() {
            continue;
        }
        if QUERY_STOP_WORDS.contains(&wl.as_str()) {
            break;
        }
        picked.push(w.trim_matches(|c: char| !c.is_alphanumeric()));
        if picked.len() >= 4 {
            break;
        }
    }
    if picked.is_empty() {
        None
    } else {
        Some(picked.join(" "))
    }
}

pub fn extract_search_query(message: &str) -> String {
    if let Some(loc) = extract_weather_location(message) {
        return format!("погода {loc}");
    }
    let normalized = message.replace('\n', " ");
    let words: Vec<&str> = normalized
        .split_whitespace()
        .filter(|w| {
            let wl = w.to_lowercase();
            !QUERY_STOP_WORDS.iter().any(|s| wl == *s)
        })
        .collect();
    let joined: String = words.join(" ");
    if joined.chars().count() > 240 {
        joined.chars().take(240).collect()
    } else {
        joined
    }
}

/// Result of auto web lookup for a chat message.
pub struct WebLookupResult {
    pub summary: String,
    pub blocked: bool,
    pub block_reason: Option<String>,
    pub error: Option<String>,
}

impl NetworkManager {
    /// DuckDuckGo + wttr.in for weather when DDG has no instant answer.
    pub async fn lookup_for_chat_message(
        &self,
        message: &str,
        chat_id: &str,
        allow_internet: bool,
        settings: &crate::settings::AppSettings,
    ) -> WebLookupResult {
        let query = extract_search_query(message);
        let chat_id_opt = Some(chat_id.to_string());

        if is_weather_query(message) {
            let location = extract_weather_location(message).unwrap_or_else(|| query.clone());
            match self
                .fetch_weather(
                    &location,
                    None,
                    chat_id_opt.clone(),
                    allow_internet,
                    settings,
                )
                .await
            {
                Ok(log) if !log.blocked => {
                    let line = log.response_preview.trim().to_string();
                    if !line.is_empty() {
                        return WebLookupResult {
                            summary: format!("Погода (wttr.in): {line}"),
                            blocked: false,
                            block_reason: None,
                            error: None,
                        };
                    }
                }
                Ok(log) => {
                    return WebLookupResult {
                        summary: String::new(),
                        blocked: true,
                        block_reason: log.block_reason,
                        error: None,
                    };
                }
                Err(e) => {
                    tracing::warn!("wttr.in weather failed: {e}");
                }
            }
        }

        match self
            .web_search(&query, None, chat_id_opt, allow_internet, settings)
            .await
        {
            Ok(log) if !log.blocked => {
                let summary = format_ddg_preview(&log.response_preview);
                WebLookupResult {
                    summary,
                    blocked: false,
                    block_reason: None,
                    error: None,
                }
            }
            Ok(log) => WebLookupResult {
                summary: String::new(),
                blocked: true,
                block_reason: log.block_reason,
                error: None,
            },
            Err(e) => WebLookupResult {
                summary: String::new(),
                blocked: false,
                block_reason: None,
                error: Some(e),
            },
        }
    }
}

/// Append web lookup context to the user message for the model.
pub fn inject_web_lookup_into_message(message: &str, lookup: &WebLookupResult) -> String {
    if lookup.blocked {
        let reason = lookup
            .block_reason
            .clone()
            .unwrap_or_else(|| "запрос заблокирован политикой сети".into());
        return format!(
            "{message}\n\n[Веб-поиск не выполнен: {reason}. Проверьте Настройки → Сеть: режим API и белый список DuckDuckGo / wttr.in.]"
        );
    }
    if let Some(err) = &lookup.error {
        return format!("{message}\n\n[Ошибка веб-поиска: {err}]");
    }
    if lookup.summary.is_empty() {
        return format!(
            "{message}\n\n[Веб-поиск выполнен, но краткого ответа нет. Дай ответ по общим знаниям и укажи, что точных данных из поиска нет.]"
        );
    }
    format!(
        "{message}\n\n[Данные из интернета]:\n{}\n\nСформируй ответ пользователю на основе этих данных.",
        lookup.summary
    )
}

/// Parse DuckDuckGo Instant Answer JSON into text for the model.
pub fn format_ddg_preview(json_text: &str) -> String {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(json_text) else {
        let trimmed = json_text.trim();
        return if trimmed.len() > 40 {
            trimmed.chars().take(1500).collect()
        } else {
            String::new()
        };
    };
    let mut parts: Vec<String> = Vec::new();
    if let Some(t) = v.get("Heading").and_then(|x| x.as_str()) {
        if !t.is_empty() {
            parts.push(t.to_string());
        }
    }
    if let Some(a) = v.get("AbstractText").and_then(|x| x.as_str()) {
        if !a.is_empty() {
            parts.push(a.to_string());
        }
    }
    if let Some(a) = v.get("Answer").and_then(|x| x.as_str()) {
        if !a.is_empty() {
            parts.push(format!("Краткий ответ: {a}"));
        }
    }
    if let Some(topics) = v.get("RelatedTopics").and_then(|x| x.as_array()) {
        for item in topics.iter().take(5) {
            if let Some(text) = item.get("Text").and_then(|x| x.as_str()) {
                if !text.is_empty() {
                    parts.push(text.to_string());
                }
            }
        }
    }
    parts.join("\n").chars().take(2000).collect()
}

pub fn ensure_ddg_api_whitelist(endpoints: &mut Vec<String>) {
    for ep in [
        "https://api.duckduckgo.com",
        "https://html.duckduckgo.com",
        "https://wttr.in",
    ] {
        if !endpoints.iter().any(|e| url_matches_whitelist(ep, e)) {
            endpoints.push(ep.into());
        }
    }
}

#[cfg(test)]
mod web_search_tests {
    use super::*;

    #[test]
    fn detects_weather_internet_query() {
        let msg = "Мне нужно узнать погоду в москве посмотри в интернете";
        assert!(message_wants_web_search(msg));
    }

    #[test]
    fn ddg_whitelist_includes_api() {
        let mut eps = vec!["https://huggingface.co".into()];
        ensure_ddg_api_whitelist(&mut eps);
        assert!(eps.iter().any(|e| e.contains("duckduckgo")));
        assert!(eps.iter().any(|e| e.contains("wttr.in")));
    }

    #[test]
    fn extracts_moscow_from_weather_message() {
        let msg = "Мне нужно узнать погоду в москве посмотри в интернете";
        assert_eq!(
            extract_weather_location(msg).as_deref(),
            Some("москве")
        );
        assert!(is_weather_query(msg));
    }

    #[test]
    fn search_query_strips_filler_words() {
        let q = extract_search_query("Мне нужно узнать погоду в москве посмотри в интернете");
        assert_eq!(q, "погода москве");
    }
}

fn url_matches_whitelist(url: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        let prefix = pattern.split('*').next().unwrap_or("");
        url.starts_with(prefix)
    } else {
        url.starts_with(pattern)
    }
}
