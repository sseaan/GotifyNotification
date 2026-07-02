import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useAppStore } from "../store/useAppStore";
import type { GotifyMessage } from "../types";

/**
 * Hook to listen for Tauri backend events:
 * - new-message: A new message arrived from Gotify
 * - unread-count-changed: The unread count was updated
 * - show-settings: User clicked "Settings" in tray menu
 */
export function useTauriEvents() {
  const addMessage = useAppStore((s) => s.addMessage);
  const setUnreadCount = useAppStore((s) => s.setUnreadCount);
  const setCurrentView = useAppStore((s) => s.setCurrentView);

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    // Listen for new messages from the backend
    listen<GotifyMessage>("new-message", (event) => {
      const msg = event.payload;
      addMessage({
        id: msg.id,
        appid: msg.appid,
        message: msg.message,
        title: msg.title,
        date: msg.date,
        read: msg.read,
      });
    }).then((unlisten) => {
      unlisteners.push(unlisten);
    });

    // Listen for unread count changes
    listen<number>("unread-count-changed", (event) => {
      setUnreadCount(event.payload);
    }).then((unlisten) => {
      unlisteners.push(unlisten);
    });

    // Listen for show-settings event from tray menu
    listen<void>("show-settings", () => {
      setCurrentView("settings");
    }).then((unlisten) => {
      unlisteners.push(unlisten);
    });

    // Listen for notification click (to focus window)
    listen<number>("notification-clicked", () => {
      // Frontend focuses; the actual window show is handled by Tauri
    }).then((unlisten) => {
      unlisteners.push(unlisten);
    });

    return () => {
      unlisteners.forEach((fn) => fn());
    };
  }, [addMessage, setUnreadCount, setCurrentView]);
}
