use vello::Scene;

use crate::gui::{
    parameter::{Parameter, Range},
    parameters::{any::PARAMS_COUNT, input_gain::InputGain, output_gain::OutputGain},
    view::View,
};

/// Describes the UI layout and widget composition.
/// Edit this file to add, remove, or reorder widgets.
pub fn compose(gui: &mut View, scene: &mut Scene, values: &[f32; PARAMS_COUNT]) {
    gui.draw_widget(
        scene,
        &Parameter::<InputGain, Range>::new(),
        values[Parameter::<InputGain, Range>::ID],
    );

    gui.draw_widget(
        scene,
        &Parameter::<OutputGain, Range>::new(),
        values[Parameter::<OutputGain, Range>::ID],
    );
}
