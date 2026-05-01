use crate::gui::app::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn Header() -> Element {
    let state = consume_context::<Signal<AppState>>();

    let model_rate = state
        .read()
        .model_rate
        .map(|r| format!("Model rate: {r:.0} Hz"))
        .unwrap_or_default();

    rsx! {
        div {
            id: "header",
            class: "flex items-center justify-between px-4 py-4 border-b bg-neutral-900 border-neutral-700",
            span {
                class: "text-amber-500 uppercase tracking-widest font-bold text-sm whitespace-nowrap",
                "Neural Amp Modeler"
            }
            span {
                id: "nam-model-rate",
                class: "text-xs text-neutral-500 tracking-wider",
                "{model_rate}"
            }
        }
    }
}
