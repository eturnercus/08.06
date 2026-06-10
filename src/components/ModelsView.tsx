import { useState } from "react";
import { useTranslation } from "react-i18next";
import i18n from "i18next";
import type { MoeInfo } from "../api/tauri";
import { HuggingFaceBrowser } from "./models/HuggingFaceBrowser";
import { LocalModelsPanel } from "./models/LocalModelsPanel";
import { useModels } from "../hooks/useModels";
import { PageIntro } from "./ui/PageIntro";
import { EmptyState } from "./ui/EmptyState";

function moeHint(moe: MoeInfo) {
  return i18n.language === "en" ? moe.hintEn : moe.hintRu;
}

export function ModelsView() {
  const { t } = useTranslation();
  const [tab, setTab] = useState<"local" | "hf" | "all">("local");
  const { models, refresh } = useModels();

  return (
    <div className="models-view">
      <PageIntro description={t("models.pageIntro")} />
      <div className="m3-tabs">
        <button type="button" className={`m3-tab ${tab === "local" ? "active" : ""}`} onClick={() => setTab("local")}>{t("models.localTab")}</button>
        <button type="button" className={`m3-tab ${tab === "hf" ? "active" : ""}`} onClick={() => setTab("hf")}>{t("models.hfTab")}</button>
        <button type="button" className={`m3-tab ${tab === "all" ? "active" : ""}`} onClick={() => { setTab("all"); refresh(); }}>{t("models.allTab")}</button>
      </div>
      {tab === "local" && <LocalModelsPanel />}
      {tab === "hf" && <HuggingFaceBrowser onDownloaded={refresh} />}
      {tab === "all" && (
        <div className="scroll-y models-all">
          {models.length === 0 && (
            <EmptyState icon="🧠" title={t("models.emptyTitle")} description={t("models.emptyDesc")} />
          )}
          {models.map((m) => (
            <div key={m.id} className="m3-card model-card">
              <strong>{m.name}</strong>
              <span className="m3-chip">{m.format}</span>
              <span className="m3-chip">{m.source}</span>
              {m.moe?.isMoe && (
                <span
                  className="m3-chip model-moe-chip"
                  title={moeHint(m.moe)}
                >
                  MoE{m.moe.expertCount ? ` ${m.moe.activeExperts ?? "?"}/${m.moe.expertCount}` : ""}
                </span>
              )}
              {m.verified && <span className="m3-chip active">✓ {t("models.verified")}</span>}
              {m.moe?.isMoe && (
                <p className="model-moe-hint">{moeHint(m.moe)}</p>
              )}
              {m.path && <div className="model-card-meta mono">{m.path}</div>}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
