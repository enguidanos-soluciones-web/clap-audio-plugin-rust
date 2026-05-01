use crate::gui::app::components::{header::Header, nam_model_selector::NamModelSelector, parameters::Parameters};
use dioxus::prelude::*;

#[component]
pub fn Layout() -> Element {
    rsx! {
        div {
            class: "flex flex-col h-full w-full bg-neutral-900",
            Header {}
            NamModelSelector {}
            Parameters {}
        }
    }
}
