use crate::{
    gestures::drag::ActiveDrag,
    gui::app::{dispatcher::Dispatcher, state::AppState},
    parameters::{Parameter, Range, blend::Blend, input_gain::InputGain, output_gain::OutputGain, tone::Tone},
    state::GuiRequest,
};
use dioxus::prelude::*;

#[component]
pub fn Parameters() -> Element {
    let state = consume_context::<Signal<AppState>>();
    let dispatcher = consume_context::<Dispatcher>();

    let mut drag = consume_context::<Signal<Option<ActiveDrag>>>();

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
                    div {
                        id: "blend",
                        class: "w-20 h-20",
                        onmousedown: {
                            let state = state.clone();
                            move |e| {
                                let coords = e.data().client_coordinates();
                                let raw = state.read().params[Parameter::<Blend, Range>::ID];
                                drag.set(ActiveDrag::from_index(Parameter::<Blend, Range>::ID, coords.x, coords.y, raw));
                            }
                        },
                        ondoubleclick: {
                            let dispatcher = dispatcher.clone();
                            move |_| dispatcher(GuiRequest::ResetParam(Parameter::<Blend, Range>::ID))
                        },
                    }
                    span { class: "text-xs font-semibold tracking-widest uppercase text-neutral-400", "Blend" }
                }

                div {
                    class: "flex flex-col items-center gap-2.5",
                    span { id: "input-gain-db", class: "text-amber-500 text-sm", "{input_db}" }
                    div {
                        id: "input-gain",
                        class: "w-20 h-20",
                        onmousedown: {
                            let state = state.clone();
                            move |e| {
                                let coords = e.data().client_coordinates();
                                let raw = state.read().params[Parameter::<InputGain, Range>::ID];
                                drag.set(ActiveDrag::from_index(Parameter::<InputGain, Range>::ID, coords.x, coords.y, raw));
                            }
                        },
                        ondoubleclick: {
                            let dispatcher = dispatcher.clone();
                            move |_| dispatcher(GuiRequest::ResetParam(Parameter::<InputGain, Range>::ID))
                        },
                    }
                    span { class: "text-xs font-semibold tracking-widest uppercase text-neutral-400", "Gain" }
                }

                div {
                    class: "flex flex-col items-center gap-2.5",
                    span { id: "tone-val", class: "text-amber-500 text-sm", "{tone_val}" }
                    div {
                        id: "tone",
                        class: "w-20 h-20",
                        onmousedown: {
                            let state = state.clone();
                            move |e| {
                                let coords = e.data().client_coordinates();
                                let raw = state.read().params[Parameter::<Tone, Range>::ID];
                                drag.set(ActiveDrag::from_index(Parameter::<Tone, Range>::ID, coords.x, coords.y, raw));
                            }
                        },
                        ondoubleclick: {
                            let dispatcher = dispatcher.clone();
                            move |_| dispatcher(GuiRequest::ResetParam(Parameter::<Tone, Range>::ID))
                        },
                    }
                    span { class: "text-xs font-semibold tracking-widest uppercase text-neutral-400", "Tone" }
                }

                div {
                    class: "flex flex-col items-center gap-2.5",
                    span { id: "output-gain-db", class: "text-amber-500 text-sm", "{output_db}" }
                    div {
                        id: "output-gain",
                        class: "w-20 h-20",
                        onmousedown: {
                            let state = state.clone();
                            move |e| {
                                let coords = e.data().client_coordinates();
                                let raw = state.read().params[Parameter::<OutputGain, Range>::ID];
                                drag.set(ActiveDrag::from_index(Parameter::<OutputGain, Range>::ID, coords.x, coords.y, raw));
                            }
                        },
                        ondoubleclick: {
                            let dispatcher = dispatcher.clone();
                            move |_| dispatcher(GuiRequest::ResetParam(Parameter::<OutputGain, Range>::ID))
                        },
                    }
                    span { class: "text-xs font-semibold tracking-widest uppercase text-neutral-400", "Master" }
                }
        }
    }
}
