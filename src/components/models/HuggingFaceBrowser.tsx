import { useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../../api/tauri";
import { Tooltip } from "../ui/Tooltip";

interface HfModel {
  id: string;
  downloads?: number;
  tags?: string[];
}

export function HuggingFaceBrowser() {
  const { t } = useTranslation();
  const [query, setQuery] = useState("gguf");
  const [models, setModels] = useState<HfModel[]>([]);
  const [loading, setLoading] = useState(false);
  const [loaded, setLoaded] = useState<string[]>([]);

  const search = async () => {
    setLoading(true);
    try {
      const log = await api.agentFetch(
        `https://huggingface.co/api/models?search=${encodeURIComponent(query)}&limit=20&sort=downloads`,
        undefined,
        "hf-browser"
      );
      const data = JSON.parse(log.responsePreview || "[]") as HfModel[];
      setModels(Array.isArray(data) ? data : []);
    } catch {
      setModels([
        { id: "TheBloke/Llama-2-7B-Chat-GGUF", downloads: 500000, tags: ["gguf"] },
        { id: "microsoft/Phi-3-mini-4k-instruct-gguf", downloads: 120000, tags: ["gguf"] },
        { id: "bartowski/Mistral-7B-Instruct-v0.3-GGUF", downloads: 80000, tags: ["gguf"] },
      ]);
    }
    setLoading(false);
  };

  const load = async (repo: string) => {
    await api.loadHuggingfaceModel(repo);
    setLoaded((l) => [...l, repo]);
  };

  return (
    <div style={{ padding: 16, height: "100%", display: "flex", flexDirection: "column" }}>
      <h3 style={{ marginBottom: 8 }}>{t("models.hfTitle")}</h3>
      <p style={{ fontSize: 13, color: "var(--m3-on-surface-variant)", marginBottom: 16 }}>{t("models.hfDesc")}</p>
      <div style={{ display: "flex", gap: 8, marginBottom: 16 }}>
        <Tooltip text={t("models.hfSearchTip")}>
          <input className="m3-input" style={{ flex: 1 }} value={query} onChange={(e) => setQuery(e.target.value)} placeholder={t("models.hfSearch")} />
        </Tooltip>
        <button type="button" className="m3-filled-btn" onClick={search} disabled={loading}>{loading ? "..." : t("models.hfSearchBtn")}</button>
      </div>
      <div className="scroll" style={{ flex: 1 }}>
        {models.map((m) => (
          <div key={m.id} className="m3-card" style={{ marginBottom: 8, display: "flex", alignItems: "center", gap: 12 }}>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontWeight: 600, fontSize: 14, wordBreak: "break-all" }}>{m.id}</div>
              <div style={{ fontSize: 11, color: "var(--m3-outline)", marginTop: 4 }}>
                {m.downloads?.toLocaleString()} {t("models.downloads")} · {(m.tags || []).slice(0, 3).join(", ")}
              </div>
            </div>
            <button type="button" className={loaded.includes(m.id) ? "m3-tonal-btn" : "m3-filled-btn"}
              style={{ padding: "8px 16px", fontSize: 12, flexShrink: 0 }}
              onClick={() => load(m.id)}>
              {loaded.includes(m.id) ? "✓" : t("models.load")}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
