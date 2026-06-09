export type UiThemePrefs = {
  theme?: string;
  fontSize?: number;
  compactMode?: boolean;
};

const LIGHT_THEMES = new Set(["light"]);

export function applyUiTheme(ui: UiThemePrefs) {
  const theme = ui.theme || "dark";
  const root = document.documentElement;
  root.setAttribute("data-theme", theme);
  root.style.colorScheme = LIGHT_THEMES.has(theme) ? "light" : "dark";
  const scale = Math.min(24, Math.max(12, ui.fontSize ?? 14));
  root.style.setProperty("--app-font-size", `${scale}px`);
  root.classList.toggle("compact-ui", Boolean(ui.compactMode));
}

export const THEME_OPTIONS = ["dark", "light", "oled", "midnight", "aurora"] as const;
