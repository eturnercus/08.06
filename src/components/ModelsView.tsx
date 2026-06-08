import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, ModelInfo } from "../api/tauri";
import { HuggingFaceBrowser } from "./models/HuggingFaceBrowser";

export function ModelsView() {
  const { t } = useTranslation();
  const [tab, setTab] = useState<"loaded" | "hf">("hf");
  const [models, setModels] = useState<ModelInfo[]>([]);

  useEffect(() => {
    api.listModels().then(setModels).catch(() => {});
  }, [tab]);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div className="m3-tabs">
        <button type="button" className={`m3-tab ${tab === "hf" ? "active" : ""}`} onClick={() => setTab("hf")}>{t("models.hfTab")}</button>
        <button type="button" className={`m3-tab ${tab === "loaded" ? "active" : ""}`} onClick={() => setTab("loaded")}>{t("models.loadedTab")}</button>
      </div>
      {tab === "hf" ? <HuggingFaceBrowser /> : (
        <div className="scroll" style={{ padding: 16 }}>
          {models.length === 0 ? <p style={{ color: "var(--m3-outline)" }}>{t("models.none")}</p> : models.map((m) => (
            <div key={m.id} className="m3-card" style={{ marginBottom: 8 }}>{m.name} <span className="m3-chip">{m.format}</span></div>
          ))}
        </div>
      )}
    </div>
  );
}
