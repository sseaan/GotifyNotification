/** A message received from the Gotify server */
export interface GotifyMessage {
  id: number;
  appid: number;
  message: string;
  title: string;
  priority: number;
  date: string;
  extras?: Record<string, unknown> | null;
  read: boolean;
  received_at: string;
}

/** Simplified message for list display */
export interface MessageListItem {
  id: number;
  appid: number;
  message: string;
  title: string;
  date: string;
  read: boolean;
}

/** Application settings */
export interface AppSettings {
  server_url: string;
  client_token: string;
  auto_start: boolean;
  start_hidden: boolean;
  minimize_to_tray: boolean;
  history_retention_days: number;
  do_not_disturb_start: string;
  do_not_disturb_end: string;
}

/** Unique app info for filtering */
export interface AppInfo {
  appid: number;
  appname: string;
}
