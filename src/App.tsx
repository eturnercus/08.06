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
  const { phase, setPhase, setSettings, addChat, settings } = useAppStore();

  useEffect(() => {
    const init = async () => {
      try {
        const s = await api.getSettings();
        setSettings(s);
        if (s.language) {
          i18n.changeLanguage(s.language);
          localStorage.setItem("neuroforge-lang", s.language);
        }
        const lang = localStorage.getItem("neuroforge-lang");
        if (s.firstRunCompleted) {
          setPhase("app");
        } else if (lang) {
          setPhase("onboarding");
        } else {
          setPhase("language");
        }
        if (useAppStore.getState().chats.length === 0) {
          addChat();
          const groups = s.agentGroups;
          if (!groups || (groups as unknown[]).length === 0) {
            const withGroup = {
              ...s,
              agentGroups: [{
                id: "default-team",
                name: "Default Research Team",
                enabled: true,
                orchestrationMode: "round_robin",
                members: [
                  { id: "coord", name: "Coordinator", role: "coordinator", modelId: "default", permissions: { internet: false, camera: false, microphone: false, screen: false, stm: true, ltm: true, canDelegate: true } },
                  { id: "research", name: "Researcher", role: "researcher", modelId: "default", permissions: { internet: true, camera: false, microphone: false, screen: false, stm: true, ltm: true, canDelegate: false } },
                ],
                sharedMemory: true,
                maxRounds: 3,
                parallelExecution: false,
              }],
            };
            await api.updateSettings(withGroup as never);
            setSettings(withGroup);
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
  if (!settings) return <div style={{ padding: 40, textAlign: "center" }}>Loading...</div>;

  return (
    <Layout>
      <MainRouter />
    </Layout>
  );
}
