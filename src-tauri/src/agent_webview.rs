use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub const WEBVIEW_LABEL: &str = "silenium-agent-browser";
pub const WEBVIEW_EVENT: &str = "agent-webview";

const BRIDGE_JS: &str = r#"
(function () {
  if (window.__sileniumBridge) return;
  window.__sileniumBridge = {
    clickNorm(x, y) {
      const px = Math.max(0, Math.min(window.innerWidth - 1, x * window.innerWidth));
      const py = Math.max(0, Math.min(window.innerHeight - 1, y * window.innerHeight));
      const el = document.elementFromPoint(px, py);
      if (!el) {
        document.title = 'SILENIUM::miss';
        return;
      }
      const label = (el.innerText || el.textContent || el.tagName || '').trim().slice(0, 80);
      document.title = 'SILENIUM::ok:' + el.tagName.toLowerCase() + ':' + label;
      el.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, view: window }));
      if (typeof el.click === 'function') el.click();
    },
    clickSelector(sel) {
      const el = document.querySelector(sel);
      if (!el) {
        document.title = 'SILENIUM::miss:' + sel;
        return;
      }
      const label = (el.innerText || el.textContent || el.tagName || '').trim().slice(0, 80);
      document.title = 'SILENIUM::ok:' + el.tagName.toLowerCase() + ':' + label;
      el.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, view: window }));
      if (typeof el.click === 'function') el.click();
    },
    scrollBy(dy) {
      window.scrollBy({ top: dy, behavior: 'instant' });
      document.title = 'SILENIUM::scroll:' + dy;
    }
  };
})();
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentWebViewState {
    pub live_enabled: bool,
    pub window_visible: bool,
    pub url: String,
    pub title: String,
    pub last_action: String,
    pub dom_mode: bool,
}

impl Default for AgentWebViewState {
    fn default() -> Self {
        Self {
            live_enabled: false,
            window_visible: false,
            url: String::new(),
            title: "Silenium Live Browser".into(),
            last_action: String::new(),
            dom_mode: false,
        }
    }
}

pub struct AgentWebView {
    inner: RwLock<AgentWebViewState>,
}

impl Default for AgentWebView {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentWebView {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(AgentWebViewState::default()),
        }
    }

    pub fn snapshot(&self) -> AgentWebViewState {
        self.inner.read().clone()
    }

    pub fn set_live_enabled(&self, app: &AppHandle, enabled: bool) {
        {
            let mut g = self.inner.write();
            g.live_enabled = enabled;
            g.dom_mode = enabled;
            if !enabled {
                g.window_visible = false;
            }
        }
        if !enabled {
            hide_webview(app);
        }
        self.emit(app);
    }

    pub async fn navigate(&self, app: &AppHandle, url: &str) -> Result<String, String> {
        let url = normalize_url(url);
        {
            let mut g = self.inner.write();
            g.url = url.clone();
            g.last_action = format!("navigate {url}");
        }
        self.emit(app);

        if !self.inner.read().live_enabled {
            return Ok(format!("Live WebView выключен (preview-only): {url}"));
        }

        ensure_window(app, &url)?;
        inject_bridge(app).await?;
        {
            let mut g = self.inner.write();
            g.window_visible = true;
            if let Some(w) = app.get_webview_window(WEBVIEW_LABEL) {
                g.title = w.title().unwrap_or_else(|_| url.clone());
            }
        }
        self.emit(app);
        Ok(format!("Live WebView: {url}"))
    }

    pub async fn click_norm(&self, app: &AppHandle, x: f32, y: f32) -> Result<String, String> {
        if !self.inner.read().live_enabled {
            return Err("Live WebView выключен — включите режим DOM в агент-браузере".into());
        }
        let w = app
            .get_webview_window(WEBVIEW_LABEL)
            .ok_or("Live WebView не открыт — сначала перейдите по URL")?;
        inject_bridge(app).await?;
        let x = x.clamp(0.0, 1.0);
        let y = y.clamp(0.0, 1.0);
        w.eval(format!("window.__sileniumBridge.clickNorm({x}, {y});"))
            .map_err(|e| e.to_string())?;
        tokio::time::sleep(Duration::from_millis(120)).await;
        let msg = read_action_result(&w);
        {
            let mut g = self.inner.write();
            g.last_action = format!("dom-click ({x:.2},{y:.2}) {msg}");
        }
        self.emit(app);
        Ok(msg)
    }

    pub async fn click_selector(&self, app: &AppHandle, selector: &str) -> Result<String, String> {
        if !self.inner.read().live_enabled {
            return Err("Live WebView выключен".into());
        }
        let sel = selector.trim();
        if sel.is_empty() {
            return Err("Пустой CSS-селектор".into());
        }
        let w = app
            .get_webview_window(WEBVIEW_LABEL)
            .ok_or("Live WebView не открыт")?;
        inject_bridge(app).await?;
        let escaped = serde_json::to_string(sel).map_err(|e| e.to_string())?;
        w.eval(format!("window.__sileniumBridge.clickSelector({escaped});"))
            .map_err(|e| e.to_string())?;
        tokio::time::sleep(Duration::from_millis(120)).await;
        let msg = read_action_result(&w);
        {
            let mut g = self.inner.write();
            g.last_action = format!("dom-selector {sel} → {msg}");
        }
        self.emit(app);
        Ok(msg)
    }

    pub async fn scroll(&self, app: &AppHandle, delta_y: f32) -> Result<String, String> {
        if !self.inner.read().live_enabled {
            return Err("Live WebView выключен".into());
        }
        let w = app
            .get_webview_window(WEBVIEW_LABEL)
            .ok_or("Live WebView не открыт")?;
        inject_bridge(app).await?;
        w.eval(format!("window.__sileniumBridge.scrollBy({delta_y});"))
            .map_err(|e| e.to_string())?;
        let msg = format!("scroll {delta_y:+.0}px");
        {
            let mut g = self.inner.write();
            g.last_action = msg.clone();
        }
        self.emit(app);
        Ok(msg)
    }

    pub fn show(&self, app: &AppHandle) -> Result<(), String> {
        if let Some(w) = app.get_webview_window(WEBVIEW_LABEL) {
            w.show().map_err(|e| e.to_string())?;
            let mut g = self.inner.write();
            g.window_visible = true;
            self.emit(app);
        }
        Ok(())
    }

    pub fn hide(&self, app: &AppHandle) {
        hide_webview(app);
        {
            let mut g = self.inner.write();
            g.window_visible = false;
        }
        self.emit(app);
    }

    fn emit(&self, app: &AppHandle) {
        let _ = app.emit(WEBVIEW_EVENT, self.snapshot());
    }
}

fn hide_webview(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(WEBVIEW_LABEL) {
        let _ = w.hide();
    }
}

fn ensure_window(app: &AppHandle, url: &str) -> Result<(), String> {
    let parsed: url::Url = url.parse().map_err(|e| format!("Некорректный URL: {e}"))?;
    if let Some(w) = app.get_webview_window(WEBVIEW_LABEL) {
        w.navigate(parsed.clone())
            .map_err(|e| e.to_string())?;
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(app, WEBVIEW_LABEL, WebviewUrl::External(parsed))
        .title("Silenium Agent Browser (Live)")
        .inner_size(960.0, 720.0)
        .min_inner_size(480.0, 360.0)
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn inject_bridge(app: &AppHandle) -> Result<(), String> {
    let w = app
        .get_webview_window(WEBVIEW_LABEL)
        .ok_or("Live WebView не найден")?;
    w.eval(BRIDGE_JS).map_err(|e| e.to_string())?;
    tokio::time::sleep(Duration::from_millis(80)).await;
    Ok(())
}

fn read_action_result(w: &tauri::WebviewWindow) -> String {
    let title = w.title().unwrap_or_default();
    if let Some(rest) = title.strip_prefix("SILENIUM::") {
        if let Some(body) = rest.strip_prefix("ok:") {
            return format!("DOM-клик: {body}");
        }
        if rest.starts_with("miss") {
            return "DOM-клик: элемент не найден".into();
        }
        return format!("DOM: {rest}");
    }
    "DOM-клик выполнен".into()
}

fn normalize_url(url: &str) -> String {
    let u = url.trim();
    if u.starts_with("http://") || u.starts_with("https://") {
        u.to_string()
    } else {
        format!("https://{u}")
    }
}

pub fn iframe_click_bridge_js() -> &'static str {
    r#"<script>
(function(){
  if (window.__sileniumIframeBridge) return;
  window.__sileniumIframeBridge = true;
  document.addEventListener('click', function(e) {
    var el = e.target;
    while (el && el.tagName !== 'A') el = el.parentElement;
    if (!el || el.tagName !== 'A') return;
    var href = el.getAttribute('href');
    if (!href || href.charAt(0) === '#') return;
    e.preventDefault();
    e.stopPropagation();
    try {
      window.parent.postMessage({
        type: 'silenium-agent-click',
        href: el.href || href,
        text: (el.innerText || el.textContent || '').trim().slice(0, 120)
      }, '*');
    } catch (_) {}
  }, true);
})();
</script>"#
}
