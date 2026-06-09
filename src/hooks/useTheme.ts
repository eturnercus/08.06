import { useEffect } from "react";
import { useAppStore } from "../store/appStore";

export function useTheme() {
  const settings = useAppStore((s) => s.settings);
  const ui = (settings?.ui ?? {}) as { theme?: string; fontSize?: number; compactMode?: boolean };

  useEffect(() => {
    const theme = ui.theme || "dark";
    document.documentElement.setAttribute("data-theme", theme);
    const scale = Math.min(24, Math.max(12, ui.fontSize ?? 14));
    document.documentElement.style.setProperty("--app-font-size", `${scale}px`);
    document.documentElement.classList.toggle("compact-ui", Boolean(ui.compactMode));
  }, [ui.theme, ui.fontSize, ui.compactMode]);
}
