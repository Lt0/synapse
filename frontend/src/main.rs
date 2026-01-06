use dioxus::prelude::*;
use dioxus::document::eval;
use dioxus_logger::tracing::Level;

fn main() {
    console_error_panic_hook::set_once();
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut clipboard_history = use_signal(|| Vec::<String>::new());

    // Effect to start monitoring and listen for events
    use_effect(move || {
        spawn(async move {
            // 1. Start monitoring (via eval to call Tauri plugin command)
            let _ = eval(r#"
                try {
                    console.log("Starting clipboard monitor...");
                    window.__TAURI__.core.invoke('plugin:clipboard|start_monitor');
                } catch (e) {
                    console.error("Failed to start monitor: " + e);
                }
            "#);

            // 2. Listen for clipboard update events
            let mut handler = eval(r#"
                const { listen } = window.__TAURI__.event;
                listen('plugin:clipboard://clipboard-monitor/update', async (event) => {
                    console.log("Clipboard update detected");
                    try {
                        // Read the clipboard text content
                        const text = await window.__TAURI__.core.invoke('plugin:clipboard|read_text');
                        console.log("Read clipboard text: " + text);
                        dioxus.send(text);
                    } catch (e) {
                        console.error("Failed to read clipboard: " + e);
                    }
                });
            "#);

            while let Ok(msg) = handler.recv().await {
                let text_val: serde_json::Value = msg;
                if let Some(text) = text_val.as_str() {
                    let text = text.trim();
                    if !text.is_empty() {
                        clipboard_history.write().push(text.to_string());
                    }
                }
            }
        });
    });

    rsx! {
        div {
            style: "width: 100vw; height: 100vh; display: flex; flex-direction: column; align-items: center; background-color: #1a1a1a; color: white; font-family: sans-serif; overflow: hidden;",
            
            // Header
            header {
                style: "width: 100%; padding: 20px; text-align: center; border-bottom: 1px solid #333; background: #222;",
                h2 { style: "color: red;", "DEBUG: FRONTEND LOADED" }
                h1 { style: "margin: 0; font-size: 24px; color: #00ffcc;", "Synapse" }
                p { style: "margin: 5px 0 0; font-size: 14px; color: #888;", "Clipboard synchronized" }
            }

            // History List
            main {
                style: "flex: 1; width: 100%; max-width: 600px; padding: 20px; overflow-y: auto;",
                if clipboard_history.read().is_empty() {
                    div { 
                        style: "text-align: center; margin-top: 50px; color: #555;",
                        "Waiting for clipboard changes..."
                    }
                } else {
                    for (i, item) in clipboard_history.read().iter().enumerate().rev() {
                        div {
                            key: "{i}",
                            style: "margin-bottom: 10px; padding: 15px; background: #2a2a2a; border-radius: 8px; border-left: 4px solid #00ffcc; word-break: break-all;",
                            "{item}"
                        }
                    }
                }
            }

            // Footer / Taskbar info
            footer {
                style: "width: 100%; padding: 10px; text-align: center; font-size: 12px; color: #444; border-top: 1px solid #333;",
                "Running in background | Tray icon active"
            }
        }
    }
}
