import { useCallback, useEffect, useState } from "react";
import { api, ModelInfo } from "../api/tauri";

export function useModels(autoLoad = true) {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [modelsDir, setModelsDir] = useState("");
  const [loading, setLoading] = useState(false);
  const [starterError, setStarterError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const [list, dir] = await Promise.all([
        api.listModels(),
        api.getModelsDirectory(),
      ]);
      const hasReal = list.some((m) => m.path && m.loaded);
      setStarterError(null);
      if (!hasReal) {
        try {
          const starter = await api.ensureStarterModel();
          if (starter) {
            const next = await api.listModels();
            setModels(next);
            setModelsDir(dir);
            setLoading(false);
            return;
          }
        } catch (e) {
          setStarterError(String(e));
        }
      }
      setModels(list);
      setModelsDir(dir);
    } catch {
      setModels([{ id: "default", name: "Default", format: "gguf", source: "builtin", loaded: true } as ModelInfo]);
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    if (autoLoad) refresh();
  }, [autoLoad, refresh]);

  return { models, modelsDir, loading, starterError, refresh };
}
