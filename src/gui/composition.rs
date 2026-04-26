use crate::gui::{
    parameter::{Parameter, Range},
    parameters::{any::PARAMS_COUNT, input_gain::InputGain, output_gain::OutputGain},
    view::View,
};
use vello::Scene;

/// Describes the UI layout and widget composition.
/// Edit this file to add, remove, or reorder widgets.
pub fn compose(view: &mut View, scene: &mut Scene, parameters_values: &[f32; PARAMS_COUNT]) {
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
