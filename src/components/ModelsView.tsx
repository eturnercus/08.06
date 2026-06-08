import { useState } from "react";
import { useTranslation } from "react-i18next";
import { HuggingFaceBrowser } from "./models/HuggingFaceBrowser";
import { LocalModelsPanel } from "./models/LocalModelsPanel";
import { useModels } from "../hooks/useModels";

export function ModelsView() {
  const { t } = useTranslation();
  const [tab, setTab] = useState<"local" | "hf" | "all">("local");
  const { models, refresh } = useModels();

  return (
    <div className="models-view">
      <div className="m3-tabs">
        <button type="button" className={`m3-tab ${tab === "local" ? "active" : ""}`} onClick={() => setTab("local")}>{t("models.localTab")}</button>
        <button type="button" className={`m3-tab ${tab === "hf" ? "active" : ""}`} onClick={() => setTab("hf")}>{t("models.hfTab")}</button>
        <button type="button" className={`m3-tab ${tab === "all" ? "active" : ""}`} onClick={() => { setTab("all"); refresh(); }}>{t("models.allTab")}</button>
      </div>
      {tab === "local" && <LocalModelsPanel />}
      {tab === "hf" && <HuggingFaceBrowser onDownloaded={refresh} />}
      {tab === "all" && (
        <div className="scroll models-all">
          {models.map((m) => (
            <div key={m.id} className="m3-card model-card">
              <strong>{m.name}</strong>
              <span className="m3-chip">{m.format}</span>
              <span className="m3-chip">{m.source}</span>
              {m.verified && <span className="m3-chip active">✓ {t("models.verified")}</span>}
              {m.path && <div className="model-card-meta mono">{m.path}</div>}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
