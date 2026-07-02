mod storage;
mod tray;
mod websocket;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Emitter, Manager, State};
use tokio::sync::Mutex;

/// Application settings stored on disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub server_url: String,
    pub client_token: String,
    pub auto_start: bool,
    pub start_hidden: bool,
    pub minimize_to_tray: bool,
    pub history_retention_days: u32,
    pub do_not_disturb_start: String, // "HH:MM"
    pub do_not_disturb_end: String,   // "HH:MM"
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            client_token: String::new(),
            auto_start: false,
            start_hidden: false,
            minimize_to_tray: true,
            history_retention_days: 7,
            do_not_disturb_start: "22:00".to_string(),
            do_not_disturb_end: "08:00".to_string(),
        }
    }
}

/// A message received from the Gotify server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GotifyMessage {
    pub id: i64,
    pub appid: i64,
    pub message: String,
    pub title: String,
    pub priority: i64,
    pub date: String,
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
    // Local tracking fields (not present in Gotify WebSocket payload)
    #[serde(default)]
    pub read: bool,
    #[serde(default)]
    pub received_at: String,
}

/// Represents what the user sees in the "received messages" list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageListItem {
    pub id: i64,
    pub appid: i64,
    pub message: String,
    pub title: String,
    pub date: String,
    pub read: bool,
}

/// Shared application state
pub struct AppState {
    pub settings: Arc<Mutex<AppSettings>>,
    pub ws_handle: Arc<Mutex<Option<websocket::WsHandle>>>,
}

/// Tauri command: save settings
#[tauri::command]
async fn save_settings(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    // Update in-memory settings
    let mut current = state.settings.lock().await;
    *current = settings.clone();

    // Persist to JSON file
    storage::save_settings(&settings).map_err(|e| e.to_string())?;

    // Update auto-start
    let app_handle_clone = app_handle.clone();
    let auto_start = settings.auto_start;
    tauri::async_runtime::spawn(async move {
        let _ = tray::set_autostart(&app_handle_clone, auto_start);
    });

    // Reconnect WebSocket with new settings
    let server_url = settings.server_url.clone();
    let client_token = settings.client_token.clone();
    if !server_url.is_empty() && !client_token.is_empty() {
        tauri::async_runtime::spawn(async move {
            let _ = websocket::reconnect(&app_handle, &server_url, &client_token).await;
        });
    }

    Ok(())
}

/// Tauri command: get settings
#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().await;
    Ok(settings.clone())
}

/// Tauri command: get all messages
#[tauri::command]
async fn get_messages(state: State<'_, AppState>) -> Result<Vec<MessageListItem>, String> {
    let settings = state.settings.lock().await;
    storage::get_messages(&settings).map_err(|e| e.to_string())
}

/// Tauri command: mark message as read
#[tauri::command]
async fn mark_message_read(state: State<'_, AppState>, message_id: i64) -> Result<(), String> {
    let settings = state.settings.lock().await;
    storage::mark_read(&settings, message_id).map_err(|e| e.to_string())
}

/// Tauri command: delete a single message
#[tauri::command]
async fn delete_message(state: State<'_, AppState>, message_id: i64) -> Result<(), String> {
    let settings = state.settings.lock().await;
    storage::delete_message(&settings, message_id).map_err(|e| e.to_string())
}

/// Tauri command: clear all messages (optionally also on server)
#[tauri::command]
async fn clear_messages(state: State<'_, AppState>, sync_server: bool) -> Result<(), String> {
    let settings = state.settings.lock().await;

    if sync_server {
        // Delete all messages from the Gotify server
        if !settings.server_url.is_empty() && !settings.client_token.is_empty() {
            let base_url = settings.server_url.trim_end_matches('/');
            let client = reqwest::Client::new();
            // Gotify DELETE /message endpoint deletes all messages
            let url = format!("{}/message", base_url);
            let _ = client
                .delete(&url)
                .header("X-Gotify-Key", &settings.client_token)
                .send()
                .await;
        }
    }

    storage::clear_messages(&settings).map_err(|e| e.to_string())
}

/// Tauri command: get unread count
#[tauri::command]
async fn get_unread_count(state: State<'_, AppState>) -> Result<i64, String> {
    let settings = state.settings.lock().await;
    storage::get_unread_count(&settings).map_err(|e| e.to_string())
}

/// Tauri command: connect to Gotify WebSocket
#[tauri::command]
async fn connect_websocket(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let settings = state.settings.lock().await;
    if settings.server_url.is_empty() || settings.client_token.is_empty() {
        return Err("Server URL and client token must be configured".to_string());
    }
    let server_url = settings.server_url.clone();
    let client_token = settings.client_token.clone();
    drop(settings);

    websocket::reconnect(&app_handle, &server_url, &client_token)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri command: disconnect WebSocket
#[tauri::command]
async fn disconnect_websocket(
    app_handle: tauri::AppHandle,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    websocket::disconnect(&app_handle).await;
    Ok(())
}

/// Tauri command: test connection to Gotify server
#[tauri::command]
async fn test_connection(server_url: String, client_token: String) -> Result<String, String> {
    let base_url = server_url.trim_end_matches('/');
    let url = format!("{}/version", base_url);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .header("X-Gotify-Key", &client_token)
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    if response.status().is_success() {
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        let version = body
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        Ok(format!("Connected! Gotify server version: {}", version))
    } else {
        Err(format!(
            "Server returned HTTP {}: {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown")
        ))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize rustls crypto provider (required for reqwest + tokio-tungstenite)
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    env_logger::init();

    // Load settings from disk (or use defaults)
    let settings = storage::load_settings().unwrap_or_default();

    let app_state = AppState {
        settings: Arc::new(Mutex::new(settings.clone())),
        ws_handle: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(Default::default(), None))
        .manage(app_state)
        .setup(move |app| {
            // Create the message window
            let start_hidden = settings.start_hidden;
            let message_window = tauri::WebviewWindowBuilder::new(
                app,
                "message",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("Nexthrum")
            .inner_size(800.0, 600.0)
            .min_inner_size(400.0, 300.0)
            .decorations(false)
            .shadow(true)
            .visible(!start_hidden)
            .build()?;

            // Intercept close → hide to tray when minimize_to_tray is enabled
            let app_handle_for_event = app.handle().clone();
            message_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let should_hide = app_handle_for_event
                        .try_state::<AppState>()
                        .map(|s| {
                            s.settings
                                .try_lock()
                                .map(|g| g.minimize_to_tray)
                                .unwrap_or(true)
                        })
                        .unwrap_or(true);
                    if should_hide {
                        api.prevent_close();
                        if let Some(w) = app_handle_for_event.get_webview_window("message") {
                            let _ = w.hide();
                        }
                    }
                }
            });

            // Build the system tray
            let _tray = tray::build_tray(app.handle())?;

            // Attempt initial WebSocket connection if configured
            if !settings.server_url.is_empty() && !settings.client_token.is_empty() {
                let handle = app.handle().clone();
                let url = settings.server_url.clone();
                let token = settings.client_token.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = websocket::connect(&handle, &url, &token).await;
                });
            }

            // Run retention cleanup on startup
            let handle = app.handle().clone();
            let settings_clone = settings.clone();
            tauri::async_runtime::spawn(async move {
                storage::cleanup_old_messages(&settings_clone);
                // Refresh tray unread indicator
                let unread = storage::get_unread_count(&settings_clone).unwrap_or(0);
                let _ = handle.emit("unread-count-changed", unread);
            });

            // Schedule periodic cleanup (every hour)
            let handle_clone = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                    let state = handle_clone.state::<AppState>();
                    let s = state.settings.lock().await;
                    storage::cleanup_old_messages(&s);
                    let unread = storage::get_unread_count(&s).unwrap_or(0);
                    drop(s);
                    let _ = handle_clone.emit("unread-count-changed", unread);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_settings,
            get_settings,
            test_connection,
            get_messages,
            mark_message_read,
            delete_message,
            clear_messages,
            get_unread_count,
            connect_websocket,
            disconnect_websocket,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
