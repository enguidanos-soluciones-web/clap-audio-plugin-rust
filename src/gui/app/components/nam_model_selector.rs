use crate::{
    gui::app::{dispatcher::Dispatcher, state::AppState},
    state::GuiRequest,
};
use dioxus::prelude::*;

#[component]
pub fn NamModelSelector() -> Element {
    let state = consume_context::<Signal<AppState>>();
    let dispatcher = consume_context::<Dispatcher>();

    let mut is_load_model_hovered = use_signal(|| false);

    let model_name = state.read().model_name.clone().unwrap_or_default();

    rsx! {
        div {
            class: "flex items-center justify-between px-4 py-2 border-b border-neutral-700 bg-neutral-800",
            span {
                id: "model-name",
                class: "text-xs text-neutral-400 truncate flex-1 mr-4",
                "{model_name}"
            }
            div {
                id: "load-model",
                class: "px-3 py-1.5 text-xs font-semibold tracking-widest uppercase border border-amber-500 cursor-pointer whitespace-nowrap",
                class: if is_load_model_hovered() { "text-neutral-900 bg-amber-500" },
                class: if !is_load_model_hovered() { "text-amber-500 bg-neutral-900" },
                onmouseenter: move |_| { is_load_model_hovered.set(true) },
                onmouseleave: move |_| { is_load_model_hovered.set(false) },
                onclick: move |_| { dispatcher(GuiRequest::OpenFileBrowser) },
                "Load Model"
            }
        }
    }
}
