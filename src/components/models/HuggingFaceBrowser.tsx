import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../../api/tauri";
import { Tooltip } from "../ui/Tooltip";
import { isTauri } from "../../api/browserFallback";

interface HfModel {
  id: string;
  downloads?: number;
  tags?: string[];
}

type DlState = "idle" | "downloading" | "ok" | "error";

export function HuggingFaceBrowser({ onDownloaded }: { onDownloaded?: () => void }) {
  const { t } = useTranslation();
  const [query, setQuery] = useState("gguf");
  const [models, setModels] = useState<HfModel[]>([]);
  const [loading, setLoading] = useState(false);
  const [searchError, setSearchError] = useState<string | null>(null);
  const [status, setStatus] = useState<Record<string, DlState>>({});
  const [messages, setMessages] = useState<Record<string, string>>({});

  const search = async () => {
    setLoading(true);
    setSearchError(null);
    try {
      if (isTauri()) {
        const data = await api.searchHuggingfaceModels(query, 20);
        setModels(data);
      } else {
        setModels([]);
        setSearchError(t("models.hfTauriOnly"));
      }
    } catch (e) {
      setSearchError(String(e));
      setModels([]);
    }
    setLoading(false);
  };

  useEffect(() => {
    if (isTauri()) search();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const download = async (repo: string) => {
    setStatus((s) => ({ ...s, [repo]: "downloading" }));
    setMessages((s) => ({ ...s, [repo]: t("models.downloading") }));
    try {
      const result = await api.downloadHuggingfaceModel(repo);
      if (result.success && result.verified) {
        setStatus((s) => ({ ...s, [repo]: "ok" }));
        setMessages((s) => ({
          ...s,
          [repo]: `${result.message} (${(result.bytesDownloaded / 1048576).toFixed(1)} MB)`,
        }));
        onDownloaded?.();
      } else {
        setStatus((s) => ({ ...s, [repo]: "error" }));
        setMessages((s) => ({ ...s, [repo]: result.message }));
      }
    } catch (e) {
      setStatus((s) => ({ ...s, [repo]: "error" }));
      setMessages((s) => ({ ...s, [repo]: String(e) }));
    }
  };

  return (
    <div className="hf-browser scroll-y">
      <div className="m3-card">
        <h3>{t("models.hfTitle")}</h3>
        <p className="form-hint">{t("models.hfDesc")}</p>
        <p className="form-hint">{t("models.hfDownloadNote")}</p>
        <div className="hf-search-row">
          <Tooltip text={t("models.hfSearchTip")}>
            <input
              className="m3-input"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t("models.hfSearch")}
              onKeyDown={(e) => e.key === "Enter" && search()}
            />
          </Tooltip>
          <button type="button" className="m3-filled-btn" onClick={search} disabled={loading}>
            {loading ? "..." : t("models.hfSearchBtn")}
          </button>
        </div>
        {searchError && <p className="field-error">{searchError}</p>}
      </div>
      {models.length === 0 && !loading && !searchError && (
        <p className="form-hint">{t("models.hfEmpty")}</p>
      )}
      {models.map((m) => {
        const st = status[m.id] ?? "idle";
        return (
          <div key={m.id} className="m3-card hf-model-card">
            <div className="hf-model-main">
              <div className="hf-model-id">{m.id}</div>
              <div className="form-hint">
                {m.downloads?.toLocaleString()} {t("models.downloads")} · {(m.tags || []).slice(0, 4).join(", ")}
              </div>
              {messages[m.id] && <div className={`hf-status hf-status-${st}`}>{messages[m.id]}</div>}
            </div>
            <button
              type="button"
              className={st === "ok" ? "m3-tonal-btn" : "m3-filled-btn"}
              disabled={st === "downloading"}
              onClick={() => download(m.id)}
            >
              {st === "downloading" ? t("models.downloading") : st === "ok" ? "✓" : t("models.download")}
            </button>
          </div>
        );
      })}
    </div>
  );
}
