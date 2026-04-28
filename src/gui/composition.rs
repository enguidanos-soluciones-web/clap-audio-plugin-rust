use crate::{
    gui::view::View,
    parameters::{Parameter, Range, any::PARAMS_COUNT, input_gain::InputGain, output_gain::OutputGain},
    state::GUIShared,
};
use vello::Scene;

pub fn compose(view: &mut View, scene: &mut Scene, state: &GUIShared, parameters_values: &[f32; PARAMS_COUNT]) {
    if let Some(nam_model_rate) = state.nam_model_rate {
        if let Some(span) = view.doc.get_element_by_id("nam-model-rate") {
            let mut mutator = view.doc.mutate();
            mutator.remove_and_drop_all_children(span);
            let text = mutator.create_text_node(&format!("Model rate: {nam_model_rate:.0} Hz"));
            mutator.append_children(span, &[text]);
        }
    }

    if let Some(span) = view.doc.get_element_by_id("input-gain-db") {
        let mut mutator = view.doc.mutate();
        mutator.remove_and_drop_all_children(span);
        let text = mutator.create_text_node(&format!("{:.1} db", parameters_values[Parameter::<InputGain, Range>::ID]));
        mutator.append_children(span, &[text]);
    }
    if let Some(span) = view.doc.get_element_by_id("output-gain-db") {
        let mut mutator = view.doc.mutate();
        mutator.remove_and_drop_all_children(span);
        let text = mutator.create_text_node(&format!("{:.1} db", parameters_values[Parameter::<OutputGain, Range>::ID]));
        mutator.append_children(span, &[text]);
    }

    view.draw_widget(
        scene,
        &Parameter::<InputGain, Range>::new(),
        parameters_values[Parameter::<InputGain, Range>::ID],
    );

    view.draw_widget(
        scene,
        &Parameter::<OutputGain, Range>::new(),
        parameters_values[Parameter::<OutputGain, Range>::ID],
    );
}
