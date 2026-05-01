use crate::{
    gui::app::state::AppState,
    parameters::{Parameter, Range, blend::Blend, input_gain::InputGain, output_gain::OutputGain, tone::Tone},
};
use dioxus::prelude::*;

#[component]
pub fn Parameters() -> Element {
    let state = consume_context::<Signal<AppState>>();

    let input_db = format!("{:.1} db", state.read().params[Parameter::<InputGain, Range>::ID]);
    let output_db = format!("{:.1} db", state.read().params[Parameter::<OutputGain, Range>::ID]);
    let tone_val = format!("{:.1}", state.read().params[Parameter::<Tone, Range>::ID] * 5.0);
    let blend_val = format!("{:.0}%", state.read().params[Parameter::<Blend, Range>::ID] * 100.0);

    rsx! {
        div {
            class: "flex-1 flex items-center justify-center gap-10",

                div {
                    class: "flex flex-col items-center gap-2.5",
                    span { id: "blend-val", class: "text-amber-500 text-sm", "{blend_val}" }
                    div { id: "blend", class: "w-20 h-20" }
                    span { class: "text-xs font-semibold tracking-widest uppercase text-neutral-400", "Blend" }
                }

                div {
                    class: "flex flex-col items-center gap-2.5",
                    span { id: "input-gain-db", class: "text-amber-500 text-sm", "{input_db}" }
                    div { id: "input-gain", class: "w-20 h-20" }
                    span { class: "text-xs font-semibold tracking-widest uppercase text-neutral-400", "Gain" }
                }

                div {
                    class: "flex flex-col items-center gap-2.5",
                    span { id: "tone-val", class: "text-amber-500 text-sm", "{tone_val}" }
                    div { id: "tone", class: "w-20 h-20" }
                    span { class: "text-xs font-semibold tracking-widest uppercase text-neutral-400", "Tone" }
                }

                div {
                    class: "flex flex-col items-center gap-2.5",
                    span { id: "output-gain-db", class: "text-amber-500 text-sm", "{output_db}" }
                    div { id: "output-gain", class: "w-20 h-20" }
                    span { class: "text-xs font-semibold tracking-widest uppercase text-neutral-400", "Master" }
                }
        }
    }
}
