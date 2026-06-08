import { useCallback, useEffect, useState } from "react";
import { api, ModelInfo } from "../api/tauri";

export function useModels(autoLoad = true) {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [modelsDir, setModelsDir] = useState("");
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const [list, dir] = await Promise.all([
        api.listModels(),
        api.getModelsDirectory(),
      ]);
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

  return { models, modelsDir, loading, refresh };
}
