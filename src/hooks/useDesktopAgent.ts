import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, DesktopAgentSnapshot } from "../api/tauri";
import { isTauri } from "../api/browserFallback";

export function useDesktopAgent() {
  const [state, setState] = useState<DesktopAgentSnapshot | null>(null);

  useEffect(() => {
    if (!isTauri()) return;
    api.getDesktopAgentState().then(setState).catch(() => {});
    const unlisten = listen<DesktopAgentSnapshot>("desktop-agent", (e) => setState(e.payload));
    return () => {
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, []);

  return state;
}
