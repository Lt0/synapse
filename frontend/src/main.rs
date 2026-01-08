use dioxus::document::eval;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;

fn main() {
    console_error_panic_hook::set_once();
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    dioxus::launch(Root);
}

#[component]
fn Root() -> Element {
    // 初始化主题并监听系统主题变化
    use_effect(move || {
        spawn(async move {
            // 检测系统主题并应用
            let _ = eval(
                r#"
                (function() {
                    // 从 localStorage 读取保存的主题偏好
                    const savedTheme = localStorage.getItem('synapse-theme');
                    let themeMode = savedTheme || 'system';
                    
                    // 检测系统主题
                    function getSystemTheme() {
                        return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
                    }
                    
                    // 应用主题
                    function applyTheme(mode) {
                        const root = document.documentElement;
                        if (mode === 'system') {
                            mode = getSystemTheme();
                        }
                        if (mode === 'dark') {
                            root.classList.add('dark');
                            root.classList.remove('light');
                        } else {
                            root.classList.add('light');
                            root.classList.remove('dark');
                        }
                    }
                    
                    // 初始应用
                    applyTheme(themeMode);
                    
                    // 监听系统主题变化
                    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
                    const handleChange = (e) => {
                        const currentTheme = localStorage.getItem('synapse-theme') || 'system';
                        if (currentTheme === 'system') {
                            applyTheme('system');
                        }
                    };
                    mediaQuery.addEventListener('change', handleChange);
                })();
                "#,
            );
        });
    });
    
    rsx! {
        link {
            rel: "stylesheet",
            href: "/theme.css"
        }
        App {}
    }
}

#[component]
fn App() -> Element {
    let mut clipboard_history = use_signal(|| Vec::<String>::new());

    // Effect to start monitoring and listen for events
    use_effect(move || {
        spawn(async move {
            // 1. Start monitoring (via eval to call Tauri plugin command)
            let _ = eval(
                r#"
                try {
                    console.log("Starting clipboard monitor...");
                    window.__TAURI__.core.invoke('plugin:clipboard|start_monitor');
                } catch (e) {
                    console.error("Failed to start monitor: " + e);
                }
            "#,
            );

            // 2. Listen for clipboard update events
            let mut handler = eval(
                r#"
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
            "#,
            );

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
            class: "container",

            // Header
            header {
                class: "header",
                img {
                    src: "/logo.svg",
                    style: "width: 64px; height: 64px; margin-bottom: 15px;"
                }
                h2 { 
                    style: "color: red; font-size: 10px; margin: 0;", 
                    "DEBUG: FRONTEND LOADED" 
                }
                h1 { "Synapse" }
                p { "Clipboard synchronized" }
            }

            // History List
            main {
                class: "main-content",
                if clipboard_history.read().is_empty() {
                    div {
                        class: "empty-state",
                        "Waiting for clipboard changes..."
                    }
                } else {
                    for (i, item) in clipboard_history.read().iter().enumerate().rev() {
                        div {
                            key: "{i}",
                            class: "history-item",
                            "{item}"
                        }
                    }
                }
            }

            // Footer / Taskbar info
            footer {
                class: "footer",
                "Running in background | Tray icon active"
            }
        }
    }
}
