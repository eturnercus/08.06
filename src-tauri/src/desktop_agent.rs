use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::agent_webview::{self, AgentWebView};
use crate::network::{FetchParams, NetworkManager};
use crate::settings::AppSettings;

pub const DESKTOP_AGENT_EVENT: &str = "desktop-agent";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualMouseState {
    pub x: f32,
    pub y: f32,
    pub visible: bool,
    pub clicking: bool,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserLink {
    pub index: usize,
    pub text: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentBrowserState {
    pub url: String,
    pub title: String,
    pub html_srcdoc: String,
    pub status: String,
    pub message: String,
    pub links: Vec<BrowserLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopAgentSnapshot {
    pub dual_mouse_enabled: bool,
    pub virtual_mouse: VirtualMouseState,
    pub browser: AgentBrowserState,
    pub webview: agent_webview::AgentWebViewState,
}

struct Inner {
    snapshot: DesktopAgentSnapshot,
    history: Vec<String>,
    history_index: usize,
}

pub struct DesktopAgent {
    inner: RwLock<Inner>,
}

impl Default for DesktopAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopAgent {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(Inner {
                snapshot: DesktopAgentSnapshot {
                    dual_mouse_enabled: false,
                    virtual_mouse: VirtualMouseState {
                        x: 0.5,
                        y: 0.5,
                        visible: false,
                        clicking: false,
                        label: String::new(),
                    },
                    browser: AgentBrowserState {
                        url: String::new(),
                        title: "Silenium Agent Browser".into(),
                        html_srcdoc: welcome_srcdoc(),
                        status: "idle".into(),
                        message: String::new(),
                        links: Vec::new(),
                    },
                    webview: agent_webview::AgentWebViewState::default(),
                },
                history: Vec::new(),
                history_index: 0,
            }),
        }
    }

    pub fn snapshot(&self, webview: &AgentWebView) -> DesktopAgentSnapshot {
        let mut snap = self.inner.read().snapshot.clone();
        snap.webview = webview.snapshot();
        snap
    }

    pub fn set_dual_mouse(&self, enabled: bool, app: &AppHandle) {
        {
            let mut g = self.inner.write();
            g.snapshot.dual_mouse_enabled = enabled;
            g.snapshot.virtual_mouse.visible = enabled;
        }
        self.emit_app_only(app);
    }

    pub fn move_mouse(&self, app: &AppHandle, x: f32, y: f32, label: Option<String>) {
        {
            let mut g = self.inner.write();
            g.snapshot.virtual_mouse.x = x.clamp(0.0, 1.0);
            g.snapshot.virtual_mouse.y = y.clamp(0.0, 1.0);
            g.snapshot.virtual_mouse.visible = g.snapshot.dual_mouse_enabled;
            if let Some(l) = label {
                g.snapshot.virtual_mouse.label = l;
            }
        }
        self.emit_app_only(app);
    }

    pub fn scroll(&self, app: &AppHandle, delta_y: f32) {
        {
            let mut g = self.inner.write();
            g.snapshot.virtual_mouse.label = format!("scroll {delta_y:+.0}");
            g.snapshot.browser.message = format!("Прокрутка страницы: {delta_y:+.0}px");
        }
        self.emit_app_only(app);
    }

    pub async fn search(
        &self,
        app: &AppHandle,
        network: &NetworkManager,
        settings: &AppSettings,
        query: &str,
        chat_id: Option<String>,
        agent_id: Option<String>,
        webview: &AgentWebView,
    ) -> Result<String, String> {
        let q = urlencoding::encode(query.trim());
        let url = format!("https://html.duckduckgo.com/html/?q={q}");
        self.navigate(app, network, settings, &url, chat_id, agent_id, webview)
            .await
    }

    pub async fn navigate(
        &self,
        app: &AppHandle,
        network: &NetworkManager,
        settings: &AppSettings,
        url: &str,
        chat_id: Option<String>,
        agent_id: Option<String>,
        webview: &AgentWebView,
    ) -> Result<String, String> {
        ensure_desktop_browser(settings)?;

        let url = normalize_url(url);
        self.set_loading(app, &url);

        let allow = internet_allowed(settings, chat_id.as_deref());
        let params = FetchParams {
            url: url.clone(),
            method: "GET".into(),
            body: None,
            agent_id: agent_id.clone(),
            chat_id: chat_id.clone(),
            allow_internet: allow,
            isolation_mode: settings.network.isolation_mode.clone(),
            api_endpoints: settings.network.api_only_endpoints.clone(),
            data_exfiltration_guard: settings.security.data_exfiltration_guard,
            audit_enabled: settings.security.audit_log_enabled,
            block_private_ips: settings.network.block_private_ips,
            network_fingerprint_check: settings.security.network_fingerprint_check,
        };

        let (status, html) = match network.fetch_page_html(params.clone()).await {
            Ok(pair) => pair,
            Err(e) => {
                self.set_error(app, &e);
                return Err(e);
            }
        };

        if status >= 400 {
            let msg = format!("HTTP {status}");
            self.set_error(app, &msg);
            return Err(msg);
        }

        if settings.security.audit_log_enabled {
            crate::settings_engine::audit_log_raw(&format!(
                "agent-browser GET {} -> {} bytes",
                url,
                html.len()
            ));
        }

        let preview: String = html.chars().take(500).collect();
        network.record_log(crate::network::NetworkRequestLog {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id,
            chat_id,
            method: "AGENT_BROWSER".into(),
            url: url.clone(),
            status: Some(status),
            request_headers: std::collections::HashMap::new(),
            response_preview: preview,
            duration_ms: 0,
            blocked: false,
            block_reason: None,
            timestamp: chrono::Utc::now(),
        });

        let html = html;
        let title = extract_title(&html).unwrap_or_else(|| url.clone());
        let links = extract_links(&html, &url);
        let srcdoc = prepare_srcdoc(&html, &url);

        {
            let mut g = self.inner.write();
            if g.history.is_empty() || g.history.last() != Some(&url) {
                let cut_at = g.history_index + 1;
                if cut_at < g.history.len() {
                    g.history.truncate(cut_at);
                }
                g.history.push(url.clone());
                g.history_index = g.history.len().saturating_sub(1);
            }
            g.snapshot.browser = AgentBrowserState {
                url: url.clone(),
                title,
                html_srcdoc: srcdoc,
                status: "ready".into(),
                message: format!("Загружено: {}", links.len()),
                links,
            };
            g.snapshot.virtual_mouse.visible = g.snapshot.dual_mouse_enabled;
            g.snapshot.virtual_mouse.label = "navigate".into();
        }
        self.emit(app, webview);

        let mut msg = format!("Открыто в агент-браузере: {url}");
        if webview.snapshot().live_enabled {
            match webview.navigate(app, &url).await {
                Ok(wv) => msg = format!("{msg} | {wv}"),
                Err(e) => msg = format!("{msg} | Live WebView: {e}"),
            }
        }
        Ok(msg)
    }

    pub async fn click_link(
        &self,
        app: &AppHandle,
        network: &NetworkManager,
        settings: &AppSettings,
        index: usize,
        chat_id: Option<String>,
        agent_id: Option<String>,
        webview: &AgentWebView,
    ) -> Result<String, String> {
        ensure_desktop_browser(settings)?;
        let (href, text, x, y) = {
            let g = self.inner.read();
            let link = g
                .snapshot
                .browser
                .links
                .iter()
                .find(|l| l.index == index)
                .ok_or_else(|| format!("Ссылка #{index} не найдена на странице"))?;
            let n = g.snapshot.browser.links.len().max(1) as f32;
            let row = (index as f32 + 0.5) / n;
            (link.href.clone(), link.text.clone(), 0.5_f32, row.clamp(0.08, 0.92))
        };

        self.animate_click(app, x, y, &text, webview);

        if webview.snapshot().live_enabled {
            if let Ok(dom) = webview.click_norm(app, x, y).await {
                return Ok(format!("ИИ-мышь + DOM: {dom}"));
            }
        }

        if href.starts_with("http://") || href.starts_with("https://") {
            self.navigate(app, network, settings, &href, chat_id, agent_id, webview)
                .await
        } else {
            Ok(format!("Клик по «{text}» (локальная ссылка: {href})"))
        }
    }

    pub async fn click_at(
        &self,
        app: &AppHandle,
        network: &NetworkManager,
        settings: &AppSettings,
        x: f32,
        y: f32,
        chat_id: Option<String>,
        agent_id: Option<String>,
        webview: &AgentWebView,
    ) -> Result<String, String> {
        ensure_desktop_browser(settings)?;
        let n = self.inner.read().snapshot.browser.links.len();
        if n == 0 {
            self.animate_click(app, x, y, "click", webview);
            if webview.snapshot().live_enabled {
                return webview.click_norm(app, x, y).await;
            }
            return Ok("Клик ИИ-мыши (нет ссылок на странице)".into());
        }
        let index = ((y * n as f32) as usize).min(n - 1);
        let link_index = {
            let g = self.inner.read();
            g.snapshot.browser.links.get(index).map(|l| l.index).unwrap_or(index)
        };
        self.click_link(app, network, settings, link_index, chat_id, agent_id, webview)
            .await
    }

    pub async fn click_selector(
        &self,
        app: &AppHandle,
        webview: &AgentWebView,
        selector: &str,
    ) -> Result<String, String> {
        self.animate_click(app, 0.5, 0.5, selector, webview);
        webview.click_selector(app, selector).await
    }

    pub fn back(&self, _app: &AppHandle) -> Option<String> {
        let url = {
            let mut g = self.inner.write();
            if g.history_index == 0 {
                return None;
            }
            g.history_index -= 1;
            g.history.get(g.history_index).cloned()
        };
        url
    }

    fn animate_click(&self, app: &AppHandle, x: f32, y: f32, label: &str, webview: &AgentWebView) {
        {
            let mut g = self.inner.write();
            g.snapshot.virtual_mouse.x = x.clamp(0.0, 1.0);
            g.snapshot.virtual_mouse.y = y.clamp(0.0, 1.0);
            g.snapshot.virtual_mouse.clicking = true;
            g.snapshot.virtual_mouse.visible = true;
            g.snapshot.virtual_mouse.label = label.to_string();
            g.snapshot.browser.message = format!("Клик ИИ-мыши: {label}");
        }
        self.emit(app, webview);
        {
            let mut g = self.inner.write();
            g.snapshot.virtual_mouse.clicking = false;
        }
        self.emit(app, webview);
    }

    fn set_loading(&self, app: &AppHandle, url: &str) {
        {
            let mut g = self.inner.write();
            g.snapshot.browser.status = "loading".into();
            g.snapshot.browser.url = url.to_string();
            g.snapshot.browser.message = format!("Загрузка {url}…");
        }
        self.emit_app_only(app);
    }

    fn set_error(&self, app: &AppHandle, msg: &str) {
        {
            let mut g = self.inner.write();
            g.snapshot.browser.status = "error".into();
            g.snapshot.browser.message = msg.to_string();
        }
        self.emit_app_only(app);
    }

    fn emit_app_only(&self, app: &AppHandle) {
        let snap = self.inner.read().snapshot.clone();
        let _ = app.emit(DESKTOP_AGENT_EVENT, snap);
    }

    fn emit(&self, app: &AppHandle, webview: &AgentWebView) {
        let _ = app.emit(DESKTOP_AGENT_EVENT, self.snapshot(webview));
    }
}

pub fn ensure_desktop_browser(settings: &AppSettings) -> Result<(), String> {
    if !settings.devices.browser_automation_enabled {
        return Err("Управление браузером отключено в Настройки → Права → Устройства".into());
    }
    if !settings.devices.desktop_control_enabled {
        return Err("Виртуальная мышь и агент-браузер отключены (Desktop control)".into());
    }
    Ok(())
}

pub fn internet_allowed(settings: &AppSettings, chat_id: Option<&str>) -> bool {
    chat_id
        .and_then(|id| settings.per_chat_overrides.get(id))
        .and_then(|o| o.allow_internet)
        .unwrap_or(settings.network.allow_internet)
}

pub fn extract_url_from_text(text: &str) -> Option<String> {
    for token in text.split_whitespace() {
        let trimmed = token
            .trim_matches(|c: char| "\"'()[]{}<>".contains(c))
            .trim_end_matches(|c: char| ",.;:!?".contains(c));
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn normalize_url(url: &str) -> String {
    let u = url.trim();
    if u.starts_with("http://") || u.starts_with("https://") {
        u.to_string()
    } else {
        format!("https://{u}")
    }
}

fn welcome_srcdoc() -> String {
    r#"<!DOCTYPE html><html><head><meta charset="utf-8"><style>
body{font-family:system-ui,sans-serif;background:#12161f;color:#e2e8f0;padding:24px;line-height:1.5}
h1{font-size:18px;color:#9d8fff}p{color:#94a3b8;font-size:13px}
</style></head><body>
<h1>Silenium Agent Browser</h1>
<p>Встроенный браузер для агентов. Ваша системная мышь не затрагивается — фиолетовый курсор принадлежит ИИ.</p>
<p>Включите «Browser automation» и «Desktop control» в настройках, затем дайте агенту разрешение «Экран» и инструменты browser_*.</p>
</body></html>"#
    .into()
}

fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<title")?;
    let after = &html[start..];
    let gt = after.find('>')? + 1;
    let end = after[gt..].find("</title>")?;
    Some(
        after[gt..gt + end]
            .trim()
            .chars()
            .take(120)
            .collect(),
    )
}

fn extract_links(html: &str, base_url: &str) -> Vec<BrowserLink> {
    let mut links = Vec::new();
    let lower = html.to_lowercase();
    let mut search_from = 0;
    let mut idx = 0;
    while let Some(rel) = lower[search_from..].find("<a ") {
        let abs = search_from + rel;
        let tag_end = html[abs..].find('>').map(|i| abs + i).unwrap_or(abs);
        let tag = &html[abs..=tag_end.min(html.len().saturating_sub(1))];
        let href = extract_attr(tag, "href").unwrap_or_default();
        if href.is_empty() || href.starts_with('#') || href.starts_with("javascript:") {
            search_from = tag_end + 1;
            continue;
        }
        let close = lower[tag_end..].find("</a>").map(|i| tag_end + i).unwrap_or(tag_end);
        let inner = strip_tags(&html[tag_end + 1..close]);
        let text = if inner.trim().is_empty() {
            href.clone()
        } else {
            inner.trim().chars().take(80).collect()
        };
        links.push(BrowserLink {
            index: idx,
            text,
            href: resolve_url(base_url, &href),
        });
        idx += 1;
        search_from = close + 4;
    }
    links
}

fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let key = format!("{name}=");
    let pos = lower.find(&key)?;
    let rest = &tag[pos + key.len()..];
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let inner = &rest[1..];
    let end = inner.find(quote)?;
    Some(inner[..end].to_string())
}

fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn resolve_url(base: &str, href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        return href.to_string();
    }
    if href.starts_with("//") {
        return format!("https:{href}");
    }
    if let Some(origin_end) = base.find("://").and_then(|i| base[i + 3..].find('/').map(|j| i + 3 + j)) {
        let origin = &base[..origin_end];
        if href.starts_with('/') {
            return format!("{origin}{href}");
        }
        let base_dir = base.rfind('/').map(|i| &base[..=i]).unwrap_or(base);
        return format!("{base_dir}{href}");
    }
    href.to_string()
}

fn prepare_srcdoc(html: &str, base_url: &str) -> String {
    let body = sanitize_html(html);
    let bridge = agent_webview::iframe_click_bridge_js();
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><base href="{base_url}"><style>
body{{font-family:system-ui,sans-serif;background:#12161f;color:#e2e8f0;padding:12px;line-height:1.45;font-size:14px}}
a{{color:#9d8fff}}img{{max-width:100%}}table{{border-collapse:collapse;width:100%}}
</style>{bridge}</head><body>{body}</body></html>"#
    )
}

fn sanitize_html(html: &str) -> String {
    let lower = html.to_lowercase();
    if let Some(body_start) = lower.find("<body") {
        if let Some(gt) = html[body_start..].find('>') {
            let content_start = body_start + gt + 1;
            if let Some(body_end) = lower[content_start..].find("</body>") {
                return strip_scripts(&html[content_start..content_start + body_end]);
            }
        }
    }
    strip_scripts(html)
}

fn strip_scripts(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let lower = html.to_lowercase();
    let mut i = 0;
    while i < html.len() {
        if lower[i..].starts_with("<script") {
            if let Some(end) = lower[i..].find("</script>") {
                i += end + 9;
                continue;
            }
        }
        if let Some(ch) = html[i..].chars().next() {
            let len = ch.len_utf8();
            out.push_str(&html[i..i + len]);
            i += len;
        } else {
            break;
        }
    }
    out
}
