import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../../api/tauri";
import { isTauri } from "../../api/browserFallback";

export function InferenceRuntimePanel({ ggufRuntime }: { ggufRuntime?: string }) {
  const { t } = useTranslation();
  const [status, setStatus] = useState<Awaited<ReturnType<typeof api.getLlamaRuntimeStatus>> | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!isTauri()) return;
    try {
      setStatus(await api.getLlamaRuntimeStatus());
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh, ggufRuntime]);

  const install = async (force = false) => {
    setLoading(true);
    setError(null);
    try {
      const next = await api.ensureLlamaRuntime(force);
      setStatus(next);
    } catch (e) {
      setError(String(e));
    }
    setLoading(false);
  };

  const mode = ggufRuntime || "silenium_core";
  const showCliInstall = mode === "llama_cli" || mode === "synaptic_auto";

  if (!isTauri()) {
    return <p className="form-hint">{t("settings.inference.runtimeBrowser")}</p>;
  }

  return (
    <div className="m3-card inference-runtime-card">
      <h3>{t("settings.inference.runtimeTitle")}</h3>
      <p className="form-hint">{t("settings.inference.runtimeDesc")}</p>
      {status?.activeEngine && (
        <p className="form-hint">
          {t("settings.inference.activeEngine")}: <strong>{t(`settings.inference.engines.${status.activeEngine}`, status.activeEngine)}</strong>
        </p>
      )}
      <ul className="runtime-status-list">
        <li>
          <span>{t("settings.inference.embeddedEngine")}</span>
          <span className={status?.embeddedAvailable ? "badge badge-green" : "badge"}>
            {status?.embeddedAvailable ? "✓" : "—"}
          </span>
        </li>
        <li>
          <span>{t("settings.inference.cliFallback")}</span>
          <span className={status?.cliReady ? "badge badge-green" : "badge badge-warn"}>
            {status?.cliReady ? "✓" : t("settings.inference.notInstalled")}
          </span>
        </li>
        {status?.version && (
          <li>
            <span>{t("settings.inference.cliVersion")}</span>
            <span className="mono">{status.version}</span>
          </li>
        )}
      </ul>
      {status?.message && <p className="form-hint">{status.message}</p>}
      {status?.cliPath && (
        <p className="form-hint mono" style={{ wordBreak: "break-all" }}>
          {status.cliPath}
        </p>
      )}
      {showCliInstall && (
        <div className="runtime-actions">
          <button type="button" className="m3-filled-btn" disabled={loading} onClick={() => install(false)}>
            {loading ? t("models.downloading") : t("settings.inference.installLlama")}
          </button>
          {status?.cliReady && (
            <button type="button" className="m3-tonal-btn" disabled={loading} onClick={() => install(true)}>
              {t("settings.inference.reinstallLlama")}
            </button>
          )}
        </div>
      )}
      {mode === "silenium_core" && (
        <p className="form-hint">{t("settings.inference.coreNoCliHint")}</p>
      )}
      {error && <p className="field-error">{error}</p>}
    </div>
  );
}
