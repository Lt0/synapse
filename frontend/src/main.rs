use dioxus::document::eval;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;

mod components;
use components::toast::ToastProvider;
use dioxus_primitives::toast::use_toast;

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
        link {
            rel: "stylesheet",
            href: asset!("/assets/dx-components-theme.css")
        }
        ToastProvider {
            App {}
        }
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
    // 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<i64>, // Unix 时间戳（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<usize>, // 内容大小（字节）
}

#[component]
fn App() -> Element {
    let clipboard_history = use_signal(|| Vec::<ClipboardItem>::new());
    
    // 删除剪贴板项的函数（通过信号更新）
    let clipboard_history_for_delete = clipboard_history;

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
                            console.log("Clipboard update event received:", JSON.stringify(event));
                            try {
                                let clipboardData = null;
                                
                                // 获取元数据
                                const timestamp = Date.now();
                                // 获取 hostname 和用户名（跨平台兼容）
                                let device = 'Unknown';
                                let username = 'Unknown';
                                try {
                                    // 使用 Tauri OS API 获取 hostname（支持所有平台）
                                    const hostname = await window.__TAURI__.os.hostname();
                                    device = hostname || 'Unknown';
                                    
                                    // 获取系统用户名（仅在桌面平台支持 shell 命令）
                                    try {
                                        const platform = await window.__TAURI__.os.platform();
                                        // 只在桌面平台（windows, macos, linux）尝试执行 shell 命令
                                        if (platform === 'windows' || platform === 'macos' || platform === 'linux') {
                                            try {
                                                const { Command } = window.__TAURI__.shell;
                                                // 使用 Command.create 创建命令
                                                const cmd = Command.create('whoami');
                                                const output = await cmd.execute();
                                                console.log("whoami output:", JSON.stringify(output));
                                                if (output.code === 0 && output.stdout) {
                                                    username = output.stdout.trim();
                                                    console.log("Got username:", username);
                                                } else {
                                                    console.log("whoami failed, code: " + output.code + ", stderr: " + output.stderr);
                                                }
                                            } catch (e) {
                                                console.error("Failed to get username via shell: " + e);
                                            }
                                        }
                                        // 在移动平台（iOS/Android），shell 命令不支持，保持 'Unknown'
                                    } catch (e) {
                                        console.log("Failed to get platform info: " + e);
                                    }
                                } catch (e) {
                                    console.log("Failed to get system info, using defaults: " + e);
                                    device = navigator.platform || 'Unknown';
                                    username = 'Unknown';
                                }
                                const getContentSize = (content) => {
                                    if (typeof content === 'string') {
                                        // 对于 base64 图片，计算实际大小
                                        if (content.startsWith('data:')) {
                                            return new Blob([content]).size;
                                        }
                                        // 对于 base64 字符串，计算解码后的大小
                                        return Math.floor(content.length * 3 / 4);
                                    }
                                    return 0;
                                };
                                
                                // 优先尝试读取图片（使用 base64 API）
                                try {
                                    const base64Image = await window.__TAURI__.core.invoke('plugin:clipboard|read_image_base64');
                                    if (base64Image && base64Image.trim() !== '') {
                                        const size = getContentSize(base64Image);
                                        clipboardData = {
                                            type: 'image',
                                            content: base64Image,
                                            mimeType: 'image/png',
                                            timestamp: timestamp,
                                            device: device,
                                            username: username,
                                            size: size
                                        };
                                        console.log("Sending image data to Dioxus, size: " + size);
                                        // dioxus.send() 会自动序列化对象
                                        dioxus.send(clipboardData);
                                        return; // 如果成功读取图片，就不读取文本了
                                    }
                                } catch (imageError) {
                                    console.log("No image in clipboard, trying text: " + imageError);
                                }
                                
                                // 如果没有图片，尝试读取文本
                                try {
                                    const text = await window.__TAURI__.core.invoke('plugin:clipboard|read_text');
                                    console.log("Read clipboard text:", text);
                                    if (text && text.trim() !== '') {
                                        const size = new Blob([text]).size;
                                        clipboardData = {
                                            type: 'text',
                                            content: text,
                                            mimeType: 'text/plain',
                                            timestamp: timestamp,
                                            device: device,
                                            username: username,
                                            size: size
                                        };
                                        console.log("Sending clipboard data to Dioxus:", JSON.stringify(clipboardData));
                                        // dioxus.send() 会自动序列化对象
                                        dioxus.send(clipboardData);
                                    }
                                } catch (textError) {
                                        console.error("Failed to read clipboard text: " + String(textError));
                                }
                            } catch (e) {
                                        console.error("Failed to read clipboard: " + String(e));
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
                                let clipboardData = null;
                                
                                // 获取元数据
                                const timestamp = Date.now();
                                // 获取 hostname 和用户名
                                let device = 'Unknown';
                                let username = 'Unknown';
                                try {
                                    // 尝试通过 Tauri 获取系统信息
                                    const hostname = await window.__TAURI__.os.hostname();
                                    device = hostname || 'Unknown';
                                    // 获取系统用户名
                                    try {
                                        const { exec } = window.__TAURI__.process;
                                        const userResult = await exec('whoami', []);
                                        username = userResult.stdout?.trim() || 'Unknown';
                                    } catch (e) {
                                        username = 'Unknown';
                                    }
                                } catch (e) {
                                    console.log("Failed to get system info, using defaults: " + e);
                                    device = navigator.platform || 'Unknown';
                                    username = 'Unknown';
                                }
                                const getContentSize = (content) => {
                                    if (typeof content === 'string') {
                                        if (content.startsWith('data:')) {
                                            return new Blob([content]).size;
                                        }
                                        return Math.floor(content.length * 3 / 4);
                                    }
                                    return 0;
                                };
                                
                                // 优先尝试读取图片（使用 base64 API）
                                try {
                                    const base64Image = await window.__TAURI__.core.invoke('plugin:clipboard|read_image_base64');
                                    if (base64Image && base64Image.trim() !== '') {
                                        const size = getContentSize(base64Image);
                                        clipboardData = {
                                            type: 'image',
                                            content: base64Image,
                                            mimeType: 'image/png',
                                            timestamp: timestamp,
                                            device: device,
                                            username: username,
                                            size: size
                                        };
                                        console.log("Clipboard image data on focus");
                                        // dioxus.send() 会自动序列化对象
                                        dioxus.send(clipboardData);
                                        return;
                                    }
                                } catch (imageError) {
                                    console.log("No image in clipboard on focus, trying text");
                                }
                                
                                // 如果没有图片，尝试读取文本
                                try {
                                    const text = await window.__TAURI__.core.invoke('plugin:clipboard|read_text');
                                    if (text && text.trim() !== '') {
                                        const size = new Blob([text]).size;
                                        clipboardData = {
                                            type: 'text',
                                            content: text,
                                            mimeType: 'text/plain',
                                            timestamp: timestamp,
                                            device: device,
                                            username: username,
                                            size: size
                                        };
                                        console.log("Clipboard data on focus:", JSON.stringify(clipboardData));
                                        // dioxus.send() 会自动序列化对象
                                        dioxus.send(clipboardData);
                                    }
                                } catch (textError) {
                                    console.error("Failed to read clipboard on focus: " + textError);
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

            let mut history = clipboard_history;
            while let Ok(msg) = handler.recv().await {
                match serde_json::from_value::<ClipboardItem>(msg) {
                    Ok(item) => {
                        let content = item.content.trim();
                        if !content.is_empty() {
                            history.write().push(item);
                        }
                    }
                    Err(e) => {
                        // 如果解析失败，记录警告（使用 console.warn 在浏览器中显示）
                        let error_msg = format!("Failed to parse clipboard item: {:?}", e);
                        let _ = eval(&format!(
                            r#"console.warn("{}");"#,
                            error_msg.replace('"', "\\\"")
                        ));
                    }
                }
            }
        });
    });

    rsx! {
        div {
            class: "container",

            // // Header
            // header {
            //     class: "header",
            //     img {
            //         src: "/logo.svg",
            //         style: "width: 64px; height: 64px; margin-bottom: 15px;"
            //     }
            //     h2 { 
            //         style: "color: red; font-size: 10px; margin: 0;", 
            //         "DEBUG: FRONTEND LOADED" 
            //     }
            //     h1 { "Synapse" }
            //     p { "Clipboard synchronized" }
            // }

            // History List
            main {
                class: "main-content",
                if clipboard_history.read().is_empty() {
                    div {
                        class: "empty-state",
                        "Waiting for clipboard changes..."
                    }
                } else {
                    for (rev_idx, item) in clipboard_history.read().iter().enumerate().rev() {
                        ClipboardItemView {
                            key: "{rev_idx}",
                            item: item.clone(),
                            rev_index: rev_idx,
                            total_len: clipboard_history.read().len(),
                            clipboard_history: clipboard_history_for_delete
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
fn ClipboardItemView(item: ClipboardItem, rev_index: usize, total_len: usize, clipboard_history: Signal<Vec<ClipboardItem>>) -> Element {
    let original_idx = total_len - 1 - rev_index;
    let toast = use_toast();
    // 格式化时间（简单的格式化）
    let time_str = item.timestamp.map(|ts| {
        // 将毫秒时间戳转换为日期时间字符串
        // 使用简单的计算方式
        let seconds = ts / 1000;
        let date = std::time::UNIX_EPOCH + std::time::Duration::from_secs(seconds as u64);
        let datetime = chrono::DateTime::<chrono::Local>::from(date);
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }).unwrap_or_else(|| "未知时间".to_string());
    
    // 格式化大小
    let size_str = item.size.map(|s| {
        if s < 1024 {
            format!("{} B", s)
        } else if s < 1024 * 1024 {
            format!("{:.2} KB", s as f64 / 1024.0)
        } else {
            format!("{:.2} MB", s as f64 / (1024.0 * 1024.0))
        }
    }).unwrap_or_else(|| "未知大小".to_string());
    
    // 格式化类型
    let type_str = match item.item_type.as_str() {
        "image" => "图片",
        "html" => "HTML",
        "file" => "文件",
        _ => "文本"
    };
    
    // 复制功能（带成功/失败提示）
    let copy_content = item.content.clone();
    let copy_type = item.item_type.clone();
    let on_copy = move |_| {
        let content = copy_content.clone();
        let item_type = copy_type.clone();
        spawn(async move {
            if item_type == "image" {
                let result = eval(&format!(
                    r#"
                    (async function() {{
                        try {{
                            await window.__TAURI__.core.invoke('plugin:clipboard|write_image_base64', {{ base64: {} }});
                            return {{ success: true, message: '图片已复制到剪贴板' }};
                        }} catch (e) {{
                            return {{ success: false, message: '复制图片失败: ' + e.message }};
                        }}
                    }})()
                    "#,
                    serde_json::to_string(&content).unwrap_or_default()
                )).await;
                
                if let Ok(result_value) = result {
                    if let Ok(result_obj) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(result_value) {
                        if let Some(success) = result_obj.get("success").and_then(|v| v.as_bool()) {
                            if let Some(message) = result_obj.get("message").and_then(|v| v.as_str()) {
                                // 使用 dioxus-primitives toast API 显示提示
                                let title = if success { "复制成功" } else { "复制失败" };
                                let msg = message.to_string();
                                let options = dioxus_primitives::toast::ToastOptions::default().description(msg);
                                if success {
                                    toast.success(title.to_string(), options);
                                } else {
                                    toast.error(title.to_string(), options);
                                }
                            }
                        }
                    }
                }
            } else {
                let text_content = content;
                let result = eval(&format!(
                    r#"
                    (async function() {{
                        try {{
                            await window.__TAURI__.core.invoke('plugin:clipboard|write_text', {{ text: {} }});
                            return {{ success: true, message: '文本已复制到剪贴板' }};
                        }} catch (e) {{
                            return {{ success: false, message: '复制文本失败: ' + e.message }};
                        }}
                    }})()
                    "#,
                    serde_json::to_string(&text_content).unwrap_or_default()
                )).await;
                
                if let Ok(result_value) = result {
                    if let Ok(result_obj) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(result_value) {
                        if let Some(success) = result_obj.get("success").and_then(|v| v.as_bool()) {
                            if let Some(message) = result_obj.get("message").and_then(|v| v.as_str()) {
                                // 使用 dioxus-primitives toast API 显示提示
                                let title = if success { "复制成功" } else { "复制失败" };
                                let msg = message.to_string();
                                let options = dioxus_primitives::toast::ToastOptions::default().description(msg);
                                if success {
                                    toast.success(title.to_string(), options);
                                } else {
                                    toast.error(title.to_string(), options);
                                }
                            }
                        }
                    }
                }
            }
        });
    };
    
    // 内容区域
    let content_area = match item.item_type.as_str() {
        "html" => {
            rsx! {
                div {
                    class: "history-item-content history-item-html",
                    dangerous_inner_html: "{item.content}"
                }
            }
        }
        "image" => {
            let mime = item.mime_type.clone().unwrap_or_else(|| "image/png".to_string());
            rsx! {
                div {
                    class: "history-item-content history-item-image",
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
                    class: if is_code { "history-item-content history-item-code" } else { "history-item-content" },
                    pre {
                        style: if is_code { "margin: 0; font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', 'Consolas', 'source-code-pro', monospace; white-space: pre-wrap; word-wrap: break-word;" } else { "margin: 0; white-space: pre-wrap; word-wrap: break-word;" },
                        "{item.content}"
                    }
                }
            }
        }
    };
    
    rsx! {
        div {
            class: "history-item",
            // 元数据头部
            div {
                class: "history-item-header",
                div {
                    class: "history-item-meta",
                    span { class: "meta-item", "时间: {time_str}" }
                    span { class: "meta-item", "设备: {item.device.as_ref().unwrap_or(&\"未知\".to_string())}" }
                    span { class: "meta-item", "用户: {item.username.as_ref().unwrap_or(&\"未知\".to_string())}" }
                    span { class: "meta-item", "大小: {size_str}" }
                    span { class: "meta-item", "类型: {type_str}" }
                }
            }
            // 内容区域
            {content_area}
            // 操作按钮
            div {
                class: "history-item-actions",
                button {
                    class: "action-button action-button-copy",
                    onclick: on_copy,
                    "复制"
                }
                button {
                    class: "action-button action-button-delete",
                    onclick: move |_| {
                        let idx = original_idx;
                        let mut history_for_delete = clipboard_history;
                        spawn(async move {
                            // 显示确认对话框
                            let confirmed = eval(r#"
                                (function() {
                                    return confirm('确定要删除这条剪贴板记录吗？');
                                })()
                            "#).await;
                            
                            if let Ok(confirmed_value) = confirmed {
                                if let Ok(true) = serde_json::from_value::<bool>(confirmed_value) {
                                    history_for_delete.write().remove(idx);
                                }
                            }
                        });
                    },
                    "删除"
                }
            }
        }
    }
}
