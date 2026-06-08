import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, ModelInfo } from "../api/tauri";

export function ModelsView() {
  const { t } = useTranslation();
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [sysInfo, setSysInfo] = useState<Record<string, unknown>>({});

  useEffect(() => {
    api.listModels().then(setModels).catch(() => {});
    api.getSystemInfo().then(setSysInfo).catch(() => {});
  }, []);

  return (
    <div className="models-view">
      <h2>{t("nav.models")}</h2>
      <div className="card" style={{ marginBottom: 16 }}>
        <div className="sys-info">
          <span>RAM: {String(sysInfo.ramLimitMb)} MB</span>
          <span>CPU: {JSON.stringify(sysInfo.cpuCores)}</span>
          <span>Threads: {String(sysInfo.threadCount)}</span>
          <span>GPU layers: {String(sysInfo.gpuLayers)}</span>
          <span>Platform: {String(sysInfo.platform)}</span>
        </div>
      </div>
      <p className="formats">{t("settings.models.formats")}</p>
      {models.length === 0 ? (
        <p className="empty">Нет загруженных моделей. Добавьте в Настройках → Модели.</p>
      ) : models.map((m) => (
        <div key={m.id} className="card model-card">
          <strong>{m.name}</strong>
          <span className="badge badge-blue">{m.format}</span>
          <span className="badge badge-green">{m.source}</span>
          {m.loaded && <span className="badge badge-green">loaded</span>}
        </div>
      ))}
      <style>{`
        .models-view { padding: 16px; overflow-y: auto; height: 100%; }
        .sys-info { display: flex; flex-wrap: wrap; gap: 12px; font-size: 13px; }
        .formats { color: var(--text2); margin: 12px 0; font-size: 13px; }
        .model-card { margin-bottom: 8px; display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
        .empty { color: var(--text2); }
      `}</style>
    </div>
  );
}
