use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            style: "width: 100vw; height: 100vh; display: flex; flex-direction: column; align-items: center; justify-content: center; background-color: #1a1a1a; color: white; font-family: sans-serif;",
            h1 { "Synapse" }
            p { "Cross-platform Clipboard Manager" }
            div {
                style: "margin-top: 20px; padding: 10px; border: 1px solid #333; border-radius: 8px;",
                "Status: Local (Demo Mode)"
            }
        }
    }
}
