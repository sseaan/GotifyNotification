import { create } from "zustand";
import type { AppSettings, MessageListItem } from "../types";

interface AppStore {
  // Settings
  settings: AppSettings;
  setSettings: (settings: AppSettings) => void;

  // Messages
  messages: MessageListItem[];
  setMessages: (messages: MessageListItem[]) => void;
  addMessage: (message: MessageListItem) => void;
  removeMessage: (id: number) => void;
  clearMessages: () => void;

  // UI State
  currentView: "messages" | "settings";
  setCurrentView: (view: "messages" | "settings") => void;
  selectedAppId: number | null;
  setSelectedAppId: (appId: number | null) => void;
  unreadCount: number;
  setUnreadCount: (count: number) => void;
  wsConnected: boolean;
  setWsConnected: (connected: boolean) => void;

  // App filters
  apps: { appid: number; appname: string }[];
  setApps: (apps: { appid: number; appname: string }[]) => void;
}

const defaultSettings: AppSettings = {
  server_url: "",
  client_token: "",
  auto_start: false,
  start_hidden: false,
  minimize_to_tray: true,
  history_retention_days: 7,
  do_not_disturb_start: "22:00",
  do_not_disturb_end: "08:00",
};

export const useAppStore = create<AppStore>((set) => ({
  settings: defaultSettings,
  setSettings: (settings) => set({ settings }),

  messages: [],
  setMessages: (messages) => {
    set({ messages });
    // Extract unique apps from messages for filtering
    const appMap = new Map<number, { appid: number; appname: string }>();
    for (const msg of messages) {
      if (!appMap.has(msg.appid)) {
        // We only have appid in MessageListItem; derive appname from messages/extras
        appMap.set(msg.appid, {
          appid: msg.appid,
          appname: `App ${msg.appid}`,
        });
      }
    }
    set({ apps: Array.from(appMap.values()) });
  },
  addMessage: (message) =>
    set((state) => {
      // Deduplicate: skip if message with this ID already exists
      if (state.messages.some((m) => m.id === message.id)) {
        return {};
      }
      const newMessages = [message, ...state.messages];
      // Update apps list
      const appMap = new Map(state.apps.map((a) => [a.appid, a]));
      if (!appMap.has(message.appid)) {
        appMap.set(message.appid, {
          appid: message.appid,
          appname: `App ${message.appid}`,
        });
      }
      return {
        messages: newMessages,
        unreadCount: state.unreadCount + (message.read ? 0 : 1),
        apps: Array.from(appMap.values()),
      };
    }),
  removeMessage: (id) =>
    set((state) => ({
      messages: state.messages.filter((m) => m.id !== id),
    })),
  clearMessages: () => set({ messages: [], apps: [] }),

  currentView: "messages",
  setCurrentView: (currentView) => set({ currentView }),
  selectedAppId: null,
  setSelectedAppId: (selectedAppId) => set({ selectedAppId }),
  unreadCount: 0,
  setUnreadCount: (unreadCount) => set({ unreadCount }),
  wsConnected: false,
  setWsConnected: (wsConnected) => set({ wsConnected }),

  apps: [],
  setApps: (apps) => set({ apps }),
}));
