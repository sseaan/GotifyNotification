use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

/// Build and configure the system tray icon
pub fn build_tray(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Create tray menu items
    let show_messages = MenuItemBuilder::with_id("show_messages", "消息列表").build(app_handle)?;
    let show_settings = MenuItemBuilder::with_id("show_settings", "设置").build(app_handle)?;
    let separator = tauri::menu::PredefinedMenuItem::separator(app_handle)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app_handle)?;

    let menu = MenuBuilder::new(app_handle)
        .item(&show_messages)
        .item(&show_settings)
        .item(&separator)
        .item(&quit)
        .build()?;

    // Create a simple tray icon programmatically (bell shape)
    let icon = create_tray_icon();

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("Nexthrum")
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "show_messages" => {
                    if let Some(window) = app.get_webview_window("message") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "show_settings" => {
                    // Emit event so the frontend shows settings
                    let _ = app.emit("show-settings", ());
                    if let Some(window) = app.get_webview_window("message") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                // On left click, show the message window
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("message") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app_handle)?;

    Ok(())
}

/// Set auto-start on Windows boot
pub fn set_autostart(_app: &AppHandle, enabled: bool) -> Result<(), String> {
    // tauri-plugin-autostart handles this automatically
    // The enable/disable commands are called from the frontend
    log::info!("Auto-start set to: {}", enabled);
    Ok(())
}

/// Load the tray icon from embedded PNG file
fn create_tray_icon() -> Image<'static> {
    let img = image::load_from_memory(include_bytes!("../icons/tray-icon.png"))
        .expect("failed to decode tray icon PNG")
        .to_rgba8();
    let (w, h) = img.dimensions();
    Image::new_owned(img.into_raw(), w, h)
}

/// Update the tray icon to show/hide the unread indicator
#[allow(dead_code)]
pub fn update_unread_indicator(app_handle: &AppHandle, has_unread: bool) {
    if let Some(tray) = app_handle.tray_by_id("main-tray") {
        if has_unread {
            // TODO: Could set a different icon with a red dot overlay
            // For now, keep the same icon
            let _ = tray.set_tooltip(Some("Nexthrum - Unread Messages"));
        } else {
            let _ = tray.set_tooltip(Some("Nexthrum"));
        }
    }
}
