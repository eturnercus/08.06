import { useEffect } from "react";
import { useAppStore } from "./store/appStore";
import { api } from "./api/tauri";
import { LanguagePicker } from "./components/LanguagePicker";
import { Onboarding } from "./components/Onboarding";
import { Layout } from "./components/Layout";
import { ChatView } from "./components/ChatView";
import { SettingsView } from "./components/SettingsView";
import { MemoryView } from "./components/MemoryView";
import { NetworkView } from "./components/NetworkView";
import { AgentsView } from "./components/AgentsView";
import { HelpView } from "./components/HelpView";
import { DevicesView } from "./components/DevicesView";
import { ModelsView } from "./components/ModelsView";
import i18n from "./i18n";
import { createDefaultAgentGroup } from "./constants/defaultAgentGroup";
import { useTheme } from "./hooks/useTheme";
import { isTauri } from "./api/browserFallback";

function MainRouter() {
  const view = useAppStore((s) => s.activeView);
  switch (view) {
    case "chats": return <ChatView />;
    case "agents": return <AgentsView />;
    case "models": return <ModelsView />;
    case "memory": return <MemoryView />;
    case "network": return <NetworkView />;
    case "devices": return <DevicesView />;
    case "settings": return <SettingsView />;
    case "help": return <HelpView />;
    default: return <ChatView />;
  }
}

export default function App() {
  const { phase, setPhase, setSettings, addChat, settings, loadChats } = useAppStore();
  useTheme();

  useEffect(() => {
    const init = async () => {
      try {
        const s = await api.getSettings();
        setSettings(s);
        loadChats();

        if (!s.firstRunCompleted || !s.language) {
          setPhase("language");
        } else {
          if (s.language) {
            i18n.changeLanguage(s.language);
            localStorage.setItem("silenium-lang", s.language);
          }
          setPhase("app");
        }

        if (useAppStore.getState().chats.length === 0) {
          addChat();
          const groups = s.agentGroups;
          if (!groups || (groups as unknown[]).length === 0) {
            const withGroup = { ...s, agentGroups: [createDefaultAgentGroup()] };
            await api.updateSettings(withGroup as never);
            setSettings(withGroup);
          }
        }

        if (isTauri()) {
          try {
            await api.ensureStarterModel();
          } catch {
            /* offline — user can download from chat properties */
          }
        }
      } catch {
        setPhase("language");
      }
    };
    init();
  }, []);

  if (phase === "language") return <LanguagePicker />;
  if (phase === "onboarding") return <Onboarding />;
  if (!settings) return <div className="app-loading">{i18n.t("app.loading")}</div>;

  return (
    <Layout>
      <MainRouter />
    </Layout>
  );
}
