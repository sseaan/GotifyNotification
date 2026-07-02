import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "./store/useAppStore";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { TitleBar } from "./components/common/TitleBar";
import { MessageWindow } from "./components/MessageWindow/MessageWindow";
import { SettingsWindow } from "./components/SettingsWindow/SettingsWindow";

function App() {
  const currentView = useAppStore((s) => s.currentView);
  const setSettings = useAppStore((s) => s.setSettings);
  const setMessages = useAppStore((s) => s.setMessages);
  const setUnreadCount = useAppStore((s) => s.setUnreadCount);

  useTauriEvents();

  useEffect(() => {
    const loadInitialData = async () => {
      try {
        const settings = await invoke("get_settings");
        setSettings(settings as Parameters<typeof setSettings>[0]);

        const messages = await invoke("get_messages");
        setMessages(messages as Parameters<typeof setMessages>[0]);

        const unread = await invoke("get_unread_count");
        setUnreadCount(unread as number);
      } catch (e) {
        console.error("Failed to load initial data:", e);
      }
    };

    loadInitialData();
  }, [setSettings, setMessages, setUnreadCount]);

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <TitleBar />
      <div className="flex-1 overflow-hidden">
        {currentView === "settings" ? <SettingsWindow /> : <MessageWindow />}
      </div>
    </div>
  );
}

export default App;
