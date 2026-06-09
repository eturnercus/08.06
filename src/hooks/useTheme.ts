import { useEffect } from "react";
import { useAppStore } from "../store/appStore";
import { applyUiTheme } from "../utils/theme";

export function useTheme() {
  const settings = useAppStore((s) => s.settings);
  const ui = settings?.ui ?? {};

  useEffect(() => {
    applyUiTheme(ui);
  }, [ui.theme, ui.fontSize, ui.compactMode]);
}
