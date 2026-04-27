use crate::{
    gui::{
        parameter::{Parameter, Range},
        parameters::{any::PARAMS_COUNT, input_gain::InputGain, output_gain::OutputGain},
        view::View,
    },
    state::GUIState,
};
use vello::Scene;

/// Describes the UI layout and widget composition.
/// Edit this file to add, remove, or reorder widgets.
pub fn compose(view: &mut View, scene: &mut Scene, state: &GUIState, parameters_values: &[f32; PARAMS_COUNT]) {
    if let Some(nam_model_rate) = state.nam_model_rate() {
        if let Some(span) = view.doc.get_element_by_id("nam-model-rate") {
            let mut mutator = view.doc.mutate();
            mutator.remove_and_drop_all_children(span);
            let text = mutator.create_text_node(&format!("Model rate: {nam_model_rate:.0} Hz"));
            mutator.append_children(span, &[text]);
        }
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
