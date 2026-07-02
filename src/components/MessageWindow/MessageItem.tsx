import { useState, useCallback, type MouseEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../store/useAppStore";
import { useI18n } from "../../i18n";
import { formatDate, truncate } from "../../utils/format";
import type { MessageListItem } from "../../types";

interface MessageItemProps {
  message: MessageListItem;
}

export function MessageItem({ message }: MessageItemProps) {
  const { t } = useI18n();
  const removeMessage = useAppStore((s) => s.removeMessage);
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
  } | null>(null);

  const handleClick = useCallback(async () => {
    // Copy message content to clipboard
    try {
      await navigator.clipboard.writeText(message.message);
    } catch {
      // Fallback for environments without clipboard API
    }

    // Mark as read if unread
    if (!message.read) {
      try {
        await invoke("mark_message_read", { messageId: message.id });
      } catch (e) {
        console.error("Failed to mark read:", e);
      }
    }
  }, [message]);

  const handleContextMenu = useCallback((e: MouseEvent) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY });
  }, []);

  const handleDelete = useCallback(async () => {
    try {
      await invoke("delete_message", { messageId: message.id });
      removeMessage(message.id);
    } catch (e) {
      console.error("Failed to delete message:", e);
    }
    setContextMenu(null);
  }, [message.id, removeMessage]);

  const handleCloseContext = useCallback(() => {
    setContextMenu(null);
  }, []);

  const isUnread = !message.read;

  return (
    <>
      <div
        onClick={handleClick}
        onContextMenu={handleContextMenu}
        className={`px-4 py-3 border-b border-[var(--color-border)] cursor-pointer transition-colors hover:bg-[var(--color-surface-hover)] ${
          isUnread ? "bg-[var(--color-surface)]" : "bg-[var(--color-bg)]"
        }`}
      >
        <div className="flex items-start justify-between gap-2">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              {isUnread && (
                <span className="w-2 h-2 rounded-full bg-[var(--color-primary)] flex-shrink-0" />
              )}
              <h3
                className={`text-sm font-medium text-[var(--color-text)] truncate`}
              >
                {message.title || "No Title"}
              </h3>
            </div>
            <p className="text-xs text-[var(--color-text-secondary)] leading-relaxed">
              {truncate(message.message, 200)}
            </p>
          </div>
          <span className="text-xs text-[var(--color-text-secondary)] whitespace-nowrap flex-shrink-0 mt-1">
            {formatDate(message.date)}
          </span>
        </div>
      </div>

      {/* Context Menu */}
      {contextMenu && (
        <>
          <div
            className="fixed inset-0 z-40"
            onClick={handleCloseContext}
            onContextMenu={(e) => {
              e.preventDefault();
              handleCloseContext();
            }}
          />
          <div
            className="fixed z-50 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-md shadow-xl py-1 min-w-[140px]"
            style={{ left: contextMenu.x, top: contextMenu.y }}
          >
            <button
              onClick={handleDelete}
              className="w-full text-left px-4 py-2 text-sm text-[var(--color-danger)] hover:bg-[var(--color-surface-hover)] transition-colors"
            >
              {t.message.delete}
            </button>
          </div>
        </>
      )}
    </>
  );
}
