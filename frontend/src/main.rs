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

// 剪贴板内容类型
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct ClipboardItem {
    #[serde(rename = "type")]
    item_type: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mime_type: Option<String>,
}

#[component]
fn App() -> Element {
    let mut clipboard_history = use_signal(|| Vec::<ClipboardItem>::new());

    // Effect to start monitoring and listen for events
    use_effect(move || {
        spawn(async move {
            // 1. Start monitoring and setup event listener
            let mut handler = eval(
                r#"
                (async function() {
                    try {
                        console.log("Starting clipboard monitor...");
                        await window.__TAURI__.core.invoke('plugin:clipboard|start_monitor');
                        console.log("Clipboard monitor started successfully");
                    } catch (e) {
                        console.error("Failed to start monitor: " + e);
                    }
                    
                    try {
                        const { listen } = window.__TAURI__.event;
                        console.log("Setting up clipboard event listener...");
                        const unlisten = await listen('plugin:clipboard://clipboard-monitor/update', async (event) => {
                            console.log("Clipboard update event received:", event);
                            try {
                                // 直接使用 Tauri API 读取剪贴板（不需要用户交互）
                                const text = await window.__TAURI__.core.invoke('plugin:clipboard|read_text');
                                console.log("Read clipboard text:", text);
                                if (text && text.trim() !== '') {
                                    const clipboardData = {
                                        type: 'text',
                                        content: text,
                                        mimeType: 'text/plain'
                                    };
                                    console.log("Sending clipboard data to Dioxus:", clipboardData);
                                    dioxus.send(clipboardData);
                                }
                            } catch (e) {
                                console.error("Failed to read clipboard: " + e);
                            }
                        });
                        console.log("Clipboard event listener set up successfully");
                    } catch (e) {
                        console.error("Failed to set up event listener: " + e);
                    }
                })();
            "#,
            );

            // 3. Also listen for window focus events to check clipboard when window gains focus
            let _ = eval(
                r#"
                (async function() {
                    try {
                        const { getCurrentWindow } = window.__TAURI__.window;
                        const currentWindow = getCurrentWindow();
                        
                        // 当窗口获得焦点时，检查剪贴板是否有新内容
                        currentWindow.onFocus(async () => {
                            console.log("Window gained focus, checking clipboard...");
                            try {
                                const text = await window.__TAURI__.core.invoke('plugin:clipboard|read_text');
                                if (text && text.trim() !== '') {
                                    const clipboardData = {
                                        type: 'text',
                                        content: text,
                                        mimeType: 'text/plain'
                                    };
                                    console.log("Clipboard data on focus:", clipboardData);
                                    dioxus.send(clipboardData);
                                }
                            } catch (e) {
                                console.error("Failed to read clipboard on focus: " + e);
                            }
                        });
                    } catch (e) {
                        console.error("Failed to set up focus listener: " + e);
                    }
                })();
            "#,
            );

            while let Ok(msg) = handler.recv().await {
                if let Ok(item) = serde_json::from_value::<ClipboardItem>(msg) {
                    let content = item.content.trim();
                    if !content.is_empty() {
                        clipboard_history.write().push(item);
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
                        ClipboardItemView {
                            key: "{i}",
                            item: item.clone()
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

#[component]
fn ClipboardItemView(item: ClipboardItem) -> Element {
    match item.item_type.as_str() {
        "html" => {
            rsx! {
                div {
                    class: "history-item history-item-html",
                    dangerous_inner_html: "{item.content}"
                }
            }
        }
        "image" => {
            let mime = item.mime_type.clone().unwrap_or_else(|| "image/png".to_string());
            rsx! {
                div {
                    class: "history-item history-item-image",
                    img {
                        src: "data:{mime};base64,{item.content}",
                        style: "max-width: 100%; height: auto; border-radius: 4px;"
                    }
                }
            }
        }
        _ => {
            // 纯文本：检测是否为代码并应用等宽字体
            let is_code = item.content.lines().any(|line| {
                line.contains("cd ") || 
                line.contains("npm ") || 
                line.contains("npx ") ||
                line.contains("git ") ||
                line.contains("sudo ") ||
                line.contains("curl ") ||
                line.contains("wget ") ||
                (line.contains("$") && line.contains(" ")) ||
                line.starts_with("#") ||
                line.starts_with("//") ||
                line.contains("function ") ||
                line.contains("const ") ||
                line.contains("let ") ||
                line.contains("import ") ||
                line.contains("export ")
            });
            
            rsx! {
                div {
                    class: if is_code { "history-item history-item-code" } else { "history-item" },
                    pre {
                        style: if is_code { "margin: 0; font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', 'Consolas', 'source-code-pro', monospace; white-space: pre-wrap; word-wrap: break-word;" } else { "margin: 0; white-space: pre-wrap; word-wrap: break-word;" },
                        "{item.content}"
                    }
                }
            }
        }
    }
}
