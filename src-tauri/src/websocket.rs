use crate::{AppSettings, AppState, GotifyMessage};
use chrono::Timelike;
use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Handle to control a running WebSocket connection
pub struct WsHandle {
    pub abort_handle: tokio::task::AbortHandle,
}

/// Connect to the Gotify WebSocket stream and spawn a read loop
pub async fn connect(
    app_handle: &AppHandle,
    server_url: &str,
    client_token: &str,
) -> Result<(), String> {
    let base_url = server_url.trim_end_matches('/');
    let ws_url = base_url
        .replace("https://", "wss://")
        .replace("http://", "ws://");
    let full_url = format!("{}/stream?token={}", ws_url, client_token);

    log::info!("Connecting to Gotify WebSocket: {}", full_url);

    // Spawn the persistent connection task with an outer reconnect loop
    let app_handle_clone = app_handle.clone();
    let url_owned = server_url.to_string();
    let token_owned = client_token.to_string();
    let full_url_clone = full_url.clone();

    let handle = tokio::spawn(async move {
        let mut current_url = full_url_clone;

        loop {
            // Attempt connection
            let ws = match connect_async(&current_url).await {
                Ok((ws, _)) => {
                    log::info!("WebSocket connected successfully");
                    ws
                }
                Err(e) => {
                    log::error!("WebSocket connection failed: {}, retrying in 5s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    // Refresh the URL from settings in case they changed
                    current_url = build_ws_url(&app_handle_clone, &url_owned, &token_owned)
                        .unwrap_or(current_url);
                    continue;
                }
            };

            let (_, read) = ws.split();

            // Read messages until connection drops
            let should_reconnect = run_read_loop(read, &app_handle_clone).await;

            if !should_reconnect {
                break; // Clean shutdown
            }

            // Prepare for reconnection
            log::info!("Reconnecting in 5s...");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            current_url =
                build_ws_url(&app_handle_clone, &url_owned, &token_owned).unwrap_or(current_url);
        }
    });

    // Store the abort handle for later disconnect
    {
        let state = app_handle.state::<AppState>();
        let mut ws = state.ws_handle.lock().await;
        *ws = Some(WsHandle {
            abort_handle: handle.abort_handle(),
        });
    }

    Ok(())
}

/// Build the WebSocket URL from stored settings (for reconnect refresh)
fn build_ws_url(app_handle: &AppHandle, default_url: &str, default_token: &str) -> Option<String> {
    let state = app_handle.try_state::<AppState>()?;
    let settings = state.settings.try_lock().ok()?;
    let url = if settings.server_url.is_empty() {
        default_url.to_string()
    } else {
        let base = settings.server_url.trim_end_matches('/');
        let ws = base
            .replace("https://", "wss://")
            .replace("http://", "ws://");
        let token = if settings.client_token.is_empty() {
            default_token
        } else {
            &settings.client_token
        };
        format!("{}/stream?token={}", ws, token)
    };
    Some(url)
}

/// Run the message read loop on a split WebSocket stream.
/// Returns `true` if the caller should reconnect, `false` for clean shutdown.
async fn run_read_loop(
    mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    app_handle: &AppHandle,
) -> bool {
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                log::info!("WS message received: {} bytes", text.len());
                match serde_json::from_str::<GotifyMessage>(&text) {
                    Ok(gotify_msg) => {
                        log::info!(
                            "Message parsed: id={}, title={}",
                            gotify_msg.id,
                            gotify_msg.title
                        );
                        handle_incoming_message(app_handle, gotify_msg).await;
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to parse WS message as GotifyMessage: {}. Raw: {}",
                            e,
                            &text[..text.len().min(200)]
                        );
                    }
                }
            }
            Ok(Message::Close(_)) => {
                log::info!("WebSocket connection closed by server");
                return true; // Reconnect
            }
            Ok(Message::Ping(_data)) => {
                // Pong handled automatically
            }
            Err(e) => {
                log::error!("WebSocket error: {}", e);
                return true; // Reconnect
            }
            _ => {}
        }
    }

    // Stream ended — reconnect
    log::info!("WebSocket stream ended");
    true
}

/// Disconnect and reconnect
pub async fn reconnect(
    app_handle: &AppHandle,
    server_url: &str,
    client_token: &str,
) -> Result<(), String> {
    disconnect(app_handle).await;
    connect(app_handle, server_url, client_token).await
}

/// Disconnect the WebSocket gracefully
pub async fn disconnect(app_handle: &AppHandle) {
    let state = app_handle.state::<AppState>();
    let mut ws = state.ws_handle.lock().await;
    if let Some(handle) = ws.take() {
        handle.abort_handle.abort();
        log::info!("WebSocket disconnected");
    }
}

/// Process an incoming Gotify message
async fn handle_incoming_message(app_handle: &AppHandle, mut msg: GotifyMessage) {
    let state = app_handle.state::<AppState>();
    let sett = state.settings.lock().await;

    let in_dnd = is_in_dnd_period(&sett);

    // Set received_at timestamp
    msg.received_at = chrono::Utc::now().to_rfc3339();
    msg.read = false;

    // Always save to database
    if let Err(e) = crate::storage::save_message(&sett, &msg) {
        log::error!("Failed to save message: {}", e);
        return;
    }

    // Clean up old messages
    crate::storage::cleanup_old_messages(&sett);

    // Get updated unread count
    let unread = crate::storage::get_unread_count(&sett).unwrap_or(0);

    // Always emit events to frontend
    let _ = app_handle.emit("new-message", &msg);
    let _ = app_handle.emit("unread-count-changed", unread);

    // Show Windows notification only outside DND hours
    if in_dnd {
        log::debug!(
            "In DND period, saved message {} without notification",
            msg.id
        );
        return;
    }

    let app_handle_clone = app_handle.clone();
    let title = msg.title.clone();
    let message = msg.message.clone();

    tauri::async_runtime::spawn(async move {
        if let Err(e) = tauri_plugin_notification::NotificationExt::notification(&app_handle_clone)
            .builder()
            .title(&title)
            .body(&message)
            .show()
        {
            log::warn!("Failed to show notification: {:?}", e);
        }
    });
}

/// Check if current time is within do-not-disturb hours
fn is_in_dnd_period(settings: &AppSettings) -> bool {
    let now = chrono::Local::now();
    let current_minutes = now.hour() * 60 + now.minute();

    let start = parse_time_to_minutes(&settings.do_not_disturb_start);
    let end = parse_time_to_minutes(&settings.do_not_disturb_end);

    if start <= end {
        current_minutes >= start && current_minutes < end
    } else {
        current_minutes >= start || current_minutes < end
    }
}

fn parse_time_to_minutes(time: &str) -> u32 {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() == 2 {
        if let (Ok(h), Ok(m)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            return h * 60 + m;
        }
    }
    0
}
