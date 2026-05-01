use crate::{
    gestures::drag::ActiveDrag,
    gui::app::{
        components::{header::Header, nam_model_selector::NamModelSelector, parameters::Parameters},
        dispatcher::Dispatcher,
    },
    state::GuiRequest,
};
use dioxus::prelude::*;
use std::time::{Duration, Instant};

const DRAG_THROTTLE: Duration = Duration::from_millis(4); // ~250fps

#[component]
pub fn Layout() -> Element {
    let dispatcher = consume_context::<Dispatcher>();

    let mut drag: Signal<Option<ActiveDrag>> = use_signal(|| None);
    let mut drag_last_dispatch: Signal<Option<Instant>> = use_signal(|| None);

    provide_context(drag);

    rsx! {
        div {
            class: "flex flex-col h-full w-full bg-neutral-900",
            onmousemove: move |e| {
                if drag.read().is_none() {
                    return;
                }

                if !e.data().held_buttons().contains(dioxus::html::input_data::MouseButton::Primary) {
                    drag.set(None);
                    drag_last_dispatch.set(None);
                    return;
                }

                let now = Instant::now();
                let should_dispatch = drag_last_dispatch
                    .read()
                    .map_or(true, |t| now.duration_since(t) >= DRAG_THROTTLE);

                if should_dispatch {
                    let coords = e.data().client_coordinates();
                    if let Some(change) = drag.read().as_ref().and_then(|a| a.on_drag(coords.x, coords.y)) {
                        dispatcher(GuiRequest::SetParam(change.index, change.value));
                        drag_last_dispatch.set(Some(now));
                    }
                }
            },
            onmouseup: move |_| {
                drag.set(None);
                drag_last_dispatch.set(None);
            },
            Header {}
            NamModelSelector {}
            Parameters {}
        }
    }
}
