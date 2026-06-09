import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, DeviceStatus } from "../api/tauri";
import { AgentBrowser } from "./devices/AgentBrowser";
import "../styles/agent-browser.css";

export function DevicesView() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<DeviceStatus | null>(null);
  const [lastCapture, setLastCapture] = useState("");

  useEffect(() => { api.getDeviceStatus().then(setStatus).catch(() => {}); }, []);

  const capture = async (type: "screen" | "audio" | "camera") => {
    const fn = type === "screen" ? api.captureScreen : type === "audio" ? api.captureAudio : api.captureCamera;
    const r = await fn() as { success: boolean; message: string };
    setLastCapture(r.message);
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
        </div>
      )}
      <div className="capture-actions">
        <button className="btn-secondary" onClick={() => capture("screen")}>🖥 {t("settings.devices.screen")}</button>
        <button className="btn-secondary" onClick={() => capture("audio")}>🎤 {t("settings.devices.microphone")}</button>
        <button className="btn-secondary" onClick={() => capture("camera")}>📷 {t("settings.devices.camera")}</button>
      </div>
      <AgentBrowser />
      {lastCapture && <div className="card" style={{ marginTop: 16 }}>{lastCapture}</div>}
      <style>{`
        .devices-view { padding: 16px; }
        .device-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px; margin: 16px 0; }
        .capture-actions { display: flex; gap: 8px; flex-wrap: wrap; }
      `}</style>
    </div>
  );
}

function StatusCard({ label, active }: { label: string; active: boolean }) {
  return (
    <div className="card">
      <div className="toggle-row"><span>{label}</span>
        <span className={`badge ${active ? "badge-green" : "badge-red"}`}>{active ? "ON" : "OFF"}</span>
      </div>
    </div>
  );
}
