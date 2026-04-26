use crate::gui::view::View;

pub enum GUIEvent {
    NamModelRateChanged(f64),
}

pub fn handle_gui_event(view: &mut View, ev: GUIEvent) {
    match ev {
        GUIEvent::NamModelRateChanged(value) => {
            if let Some(span) = view.doc.get_element_by_id("nam-model-rate") {
                let mut mutator = view.doc.mutate();
                mutator.remove_and_drop_all_children(span);
                let text = mutator.create_text_node(&format!("Model rate: {value:.0} Hz"));
                mutator.append_children(span, &[text]);
            };
        }
    };
}
