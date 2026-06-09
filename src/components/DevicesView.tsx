import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, CaptureResult, DeviceStatus } from "../api/tauri";
import { AgentBrowser } from "./devices/AgentBrowser";
import "../styles/agent-browser.css";

export function DevicesView() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<DeviceStatus | null>(null);
  const [lastCapture, setLastCapture] = useState<CaptureResult | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    api.getDeviceStatus().then(setStatus).catch(() => {});
  }, []);

  const runCapture = async (type: "screen" | "audio" | "camera" | "ocr" | "stt") => {
    setBusy(true);
    try {
      const fn =
        type === "screen"
          ? api.captureScreen
          : type === "audio"
            ? api.captureAudio
            : type === "camera"
              ? api.captureCamera
              : type === "ocr"
                ? api.ocrScreen
                : api.transcribeAudio;
      const r = (await fn()) as CaptureResult;
      setLastCapture(r);
    } catch (e) {
      setLastCapture({ success: false, message: String(e) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="devices-view">
      <h2>{t("nav.devices")}</h2>
      {status && (
        <div className="device-grid">
          <StatusCard label={t("settings.devices.camera")} active={status.cameraAvailable} />
          <StatusCard label={t("settings.devices.microphone")} active={status.microphoneAvailable} />
          <StatusCard label={t("settings.devices.screen")} active={status.screenCaptureAvailable} />
          <StatusCard label={t("settings.devices.virtualDisplay")} active={status.virtualDisplayActive} />
          <StatusCard label={t("settings.devices.ocr")} active={!!status.ocrAvailable} />
          <StatusCard label={t("settings.devices.transcribe")} active={!!status.sttAvailable} />
        </div>
      )}
      <div className="capture-actions">
        <button className="btn-secondary" disabled={busy} onClick={() => runCapture("screen")}>
          🖥 {t("settings.devices.screen")}
        </button>
        <button className="btn-secondary" disabled={busy} onClick={() => runCapture("ocr")}>
          🔤 {t("devices.capture.ocr")}
        </button>
        <button className="btn-secondary" disabled={busy} onClick={() => runCapture("audio")}>
          🎤 {t("settings.devices.microphone")}
        </button>
        <button className="btn-secondary" disabled={busy} onClick={() => runCapture("stt")}>
          📝 {t("devices.capture.stt")}
        </button>
        <button className="btn-secondary" disabled={busy} onClick={() => runCapture("camera")}>
          📷 {t("settings.devices.camera")}
        </button>
      </div>

      {lastCapture && (
        <div className={`card capture-result${lastCapture.success ? "" : " capture-error"}`}>
          <p>{lastCapture.message}</p>
          {lastCapture.dataBase64 && lastCapture.mimeType?.startsWith("image/") && (
            <img
              className="capture-preview"
              src={`data:${lastCapture.mimeType};base64,${lastCapture.dataBase64}`}
              alt={t("devices.capture.preview")}
            />
          )}
          {lastCapture.text && (
            <pre className="capture-text scroll-y">{lastCapture.text}</pre>
          )}
        </div>
      )}

      <AgentBrowser />

      <style>{`
        .devices-view { padding: 16px; }
        .device-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px; margin: 16px 0; }
        .capture-actions { display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 16px; }
        .capture-result { margin-bottom: 16px; }
        .capture-error { border-color: var(--m3-error, #cf6679); }
        .capture-preview { max-width: 100%; max-height: 320px; margin-top: 12px; border-radius: 8px; border: 1px solid var(--border-subtle, #2a3140); }
        .capture-text { margin-top: 12px; max-height: 200px; padding: 12px; background: var(--surface-dim, #0f1319); border-radius: 8px; font-size: 13px; white-space: pre-wrap; }
      `}</style>
    </div>
  );
}

function StatusCard({ label, active }: { label: string; active: boolean }) {
  return (
    <div className="card">
      <div className="toggle-row">
        <span>{label}</span>
        <span className={`badge ${active ? "badge-green" : "badge-red"}`}>{active ? "ON" : "OFF"}</span>
      </div>
    </div>
  );
}
