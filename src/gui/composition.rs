use crate::{
    gui::view::View,
    parameters::{Parameter, Range, any::PARAMS_COUNT, input_gain::InputGain, output_gain::OutputGain, tone::Tone},
    state::GUIShared,
};
use vello::Scene;

/// Mutates DOM text nodes only when values actually changed. Returns true if any node was mutated.
pub fn update_dom(
    view: &mut View,
    state: &GUIShared,
    parameters_values: &[f64; PARAMS_COUNT],
    prev_state: &GUIShared,
    prev_params: &[f64; PARAMS_COUNT],
) -> bool {
    let mut dirty = false;

    if state.nam_model_rate != prev_state.nam_model_rate {
        if let Some(nam_model_rate) = state.nam_model_rate {
            if let Some(span) = view.doc.get_element_by_id("nam-model-rate") {
                let mut mutator = view.doc.mutate();
                mutator.remove_and_drop_all_children(span);
                let text = mutator.create_text_node(&format!("Model rate: {nam_model_rate:.0} Hz"));
                mutator.append_children(span, &[text]);
                dirty = true;
            }
        }
    }

    let input_id = Parameter::<InputGain, Range>::ID;
    if parameters_values[input_id] != prev_params[input_id] {
        if let Some(span) = view.doc.get_element_by_id("input-gain-db") {
            let mut mutator = view.doc.mutate();
            mutator.remove_and_drop_all_children(span);
            let text = mutator.create_text_node(&format!("{:.1} db", parameters_values[input_id]));
            mutator.append_children(span, &[text]);
            dirty = true;
        }
    }

    let output_id = Parameter::<OutputGain, Range>::ID;
    if parameters_values[output_id] != prev_params[output_id] {
        if let Some(span) = view.doc.get_element_by_id("output-gain-db") {
            let mut mutator = view.doc.mutate();
            mutator.remove_and_drop_all_children(span);
            let text = mutator.create_text_node(&format!("{:.1} db", parameters_values[output_id]));
            mutator.append_children(span, &[text]);
            dirty = true;
        }
    }

    let tone_id = Parameter::<Tone, Range>::ID;
    if parameters_values[tone_id] != prev_params[tone_id] {
        if let Some(span) = view.doc.get_element_by_id("tone-val") {
            let mut mutator = view.doc.mutate();
            mutator.remove_and_drop_all_children(span);
            let text = mutator.create_text_node(&format!("{:.1}", parameters_values[tone_id] * 5.));
            mutator.append_children(span, &[text]);
            dirty = true;
        }
    }

    dirty
}

/// Draws widget shapes into the Vello scene (pure vector, no DOM involved).
pub fn draw_widgets(view: &mut View, scene: &mut Scene, parameters_values: &[f64; PARAMS_COUNT]) {
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

    view.draw_widget(
        scene,
        &Parameter::<Tone, Range>::new(),
        parameters_values[Parameter::<Tone, Range>::ID],
    );
}
