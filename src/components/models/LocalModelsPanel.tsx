import { useState } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { api, ModelInfo } from "../../api/tauri";
import { useModels } from "../../hooks/useModels";
import { Tooltip } from "../ui/Tooltip";

export function LocalModelsPanel() {
  const { t } = useTranslation();
  const { models, modelsDir, refresh, loading } = useModels();
  const [msg, setMsg] = useState("");
  const local = models.filter((m) => m.source === "local" || m.source === "huggingface");

  const browseFile = async () => {
    const selected = await open({
      multiple: false,
      filters: [{
        name: "AI Models",
        extensions: ["gguf", "ggml", "onnx", "safetensors", "bin", "pt", "pth"],
      }],
    });
    if (!selected || typeof selected !== "string") return;
    try {
      const name = selected.split(/[/\\]/).pop() || "model";
      await api.loadModel(selected, name);
      setMsg(t("models.loadedOk", { name }));
      refresh();
    } catch (e) {
      setMsg(String(e));
    }
  };

  const scanFolder = async () => {
    const found = await api.scanLocalModels();
    setMsg(t("models.scanResult", { count: found.length }));
    refresh();
  };

  const verify = async (m: ModelInfo) => {
    const ok = await api.verifyModel(m.id);
    setMsg(ok ? t("models.verifyOk", { name: m.name }) : t("models.verifyFail", { name: m.name }));
    refresh();
  };

  const openFolder = async () => {
    const selected = await open({ directory: true, multiple: false, defaultPath: modelsDir });
    if (!selected || typeof selected !== "string") return;
    setMsg(selected);
  };

  const copyPath = async () => {
    try {
      await navigator.clipboard.writeText(modelsDir);
      setMsg(t("models.pathCopied"));
    } catch {
      setMsg(modelsDir);
    }
  };

  const fmtSize = (b?: number) => (b ? `${(b / 1048576).toFixed(1)} MB` : "—");

  return (
    <div className="models-local scroll">
      <div className="m3-card models-dir-card">
        <h3>{t("models.localTitle")}</h3>
        <p className="form-hint">{t("models.localDesc")}</p>
        <div className="models-dir-path mono">{modelsDir}</div>
        <p className="form-hint">{t("models.localHint")}</p>
        <div className="models-actions">
          <button type="button" className="m3-filled-btn" onClick={browseFile}>{t("models.browse")}</button>
          <button type="button" className="m3-outlined-btn" onClick={openFolder}>{t("models.openFolder")}</button>
          <button type="button" className="m3-outlined-btn" onClick={scanFolder} disabled={loading}>{t("models.scan")}</button>
          <button type="button" className="m3-tonal-btn" onClick={copyPath}>{t("models.copyPath")}</button>
        </div>
        {msg && <p className="models-msg">{msg}</p>}
      </div>

      <h4 className="models-section-title">{t("models.loadedTab")} ({local.length})</h4>
      {local.length === 0 ? (
        <p className="form-hint">{t("models.noneLocal")}</p>
      ) : local.map((m) => (
        <div key={m.id} className="m3-card model-card">
          <div className="model-card-main">
            <strong>{m.name}</strong>
            <span className="m3-chip">{m.format}</span>
            <span className="m3-chip">{m.source}</span>
            {m.verified && <span className="m3-chip active">✓</span>}
          </div>
          <div className="model-card-meta mono">{m.path || "—"}</div>
          <div className="model-card-meta">{fmtSize(m.sizeBytes)}</div>
          <button type="button" className="m3-outlined-btn model-verify-btn" onClick={() => verify(m)}>
            {t("models.verify")}
          </button>
        </div>
      ))}
    </div>
  );
}
