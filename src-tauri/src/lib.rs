use tauri::{
    menu::{Menu, MenuEvent, MenuItem},
    tray::{TrayIcon, TrayIconBuilder, TrayIconEvent},
    Listener, Manager, WindowEvent,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard::init())
        .setup(|app| {
            // 1. Create Tray Menu
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).expect("failed to create quit item");
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>).expect("failed to create show item");
            let menu = Menu::with_items(app, &[&show_i, &quit_i]).expect("failed to create menu");

            // 2. Build Tray Icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app: &tauri::AppHandle, event: MenuEvent| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            #[cfg(target_os = "macos")]
                            let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray: &TrayIcon, event: TrayIconEvent| {
                    if let TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            #[cfg(target_os = "macos")]
                            let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
                        }
                    }
                })
                .build(app).expect("failed to build tray");

            // 3. Setup Clipboard Monitor
            // Note: Monitoring is currently best started from the frontend due to plugin API constraints in Rust for V2.
            // However, we register the listener in Rust to demonstrate backend reaction.
            app.listen("plugin:clipboard://text-changed", move |event: tauri::Event| {
                log::info!("Clipboard text changed: {:?}", event.payload());
            });

            Ok(())
        })
        .on_window_event(|window: &tauri::Window, event: &WindowEvent| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Hide window instead of closing
                let _ = window.hide();
                api.prevent_close();
                #[cfg(target_os = "macos")]
                let _ = window.app_handle().set_activation_policy(tauri::ActivationPolicy::Accessory);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
