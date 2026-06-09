import { useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../../api/tauri";
import { useDesktopAgent } from "../../hooks/useDesktopAgent";
import { useAppStore } from "../../store/appStore";

export function AgentBrowser() {
  const { t } = useTranslation();
  const state = useDesktopAgent();
  const settings = useAppStore((s) => s.settings);
  const [urlInput, setUrlInput] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  const enabled =
    (settings?.devices?.desktopControlEnabled as boolean) &&
    (settings?.devices?.browserAutomationEnabled as boolean);

  const browser = state?.browser;
  const mouse = state?.virtualMouse;

  const navigate = async (url: string) => {
    setBusy(true);
    setError("");
    try {
      await api.browserNavigateInApp(url);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const search = async () => {
    const q = urlInput.trim();
    if (!q) return;
    setBusy(true);
    setError("");
    try {
      await api.browserSearchInApp(q);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  if (!enabled) {
    return (
      <div className="agent-browser card">
        <h3>{t("devices.agentBrowser.title")}</h3>
        <p className="field-hint">{t("devices.agentBrowser.disabledHint")}</p>
      </div>
    );
  }

  return (
    <div className="agent-browser card">
      <div className="agent-browser-head">
        <div>
          <h3>{t("devices.agentBrowser.title")}</h3>
          <p className="field-hint">{t("devices.agentBrowser.dualMouseHint")}</p>
        </div>
        <span className={`badge ${state?.dualMouseEnabled ? "badge-purple" : "badge-red"}`}>
          {state?.dualMouseEnabled ? t("devices.agentBrowser.aiMouseOn") : t("devices.agentBrowser.aiMouseOff")}
        </span>
      </div>

      <div className="agent-browser-bar">
        <input
          className="m3-input"
          value={urlInput}
          onChange={(e) => setUrlInput(e.target.value)}
          placeholder={t("devices.agentBrowser.urlPh")}
          onKeyDown={(e) => e.key === "Enter" && (urlInput.includes(".") ? navigate(urlInput) : search())}
        />
        <button type="button" className="m3-tonal-btn" disabled={busy} onClick={() => navigate(urlInput)}>
          {t("devices.agentBrowser.go")}
        </button>
        <button type="button" className="m3-tonal-btn" disabled={busy} onClick={search}>
          {t("devices.agentBrowser.search")}
        </button>
      </div>

      {error && <div className="agent-browser-error">{error}</div>}

      <div className="agent-browser-frame-wrap">
        <div className="agent-browser-status">
          <span className="mono">{browser?.url || "—"}</span>
          <span className={`badge ${browser?.status === "ready" ? "badge-green" : browser?.status === "error" ? "badge-red" : "badge-blue"}`}>
            {browser?.status ?? "idle"}
          </span>
        </div>
        <div className="agent-browser-viewport">
          {browser?.htmlSrcdoc ? (
            <iframe
              title="agent-browser"
              className="agent-browser-iframe"
              sandbox="allow-same-origin"
              srcDoc={browser.htmlSrcdoc}
            />
          ) : (
            <div className="agent-browser-placeholder">{t("devices.agentBrowser.empty")}</div>
          )}
          {mouse?.visible && (
            <div
              className={`agent-virtual-cursor${mouse.clicking ? " clicking" : ""}`}
              style={{ left: `${mouse.x * 100}%`, top: `${mouse.y * 100}%` }}
              title={mouse.label || t("devices.agentBrowser.aiCursor")}
            >
              <span className="agent-cursor-icon">◎</span>
              {mouse.label && <span className="agent-cursor-label">{mouse.label}</span>}
            </div>
          )}
        </div>
        {browser?.links && browser.links.length > 0 && (
          <div className="agent-browser-links scroll-y">
            <div className="label">{t("devices.agentBrowser.links")}</div>
            {browser.links.slice(0, 12).map((link) => (
              <button
                key={link.index}
                type="button"
                className="agent-link-btn"
                disabled={busy}
                onClick={async () => {
                  setBusy(true);
                  try {
                    await api.browserClickInApp({ linkIndex: link.index });
                  } catch (e) {
                    setError(String(e));
                  } finally {
                    setBusy(false);
                  }
                }}
              >
                <span>{link.text || link.href}</span>
                <span className="mono">{link.href.slice(0, 48)}</span>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
