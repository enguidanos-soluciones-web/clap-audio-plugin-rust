use vello::Scene;

use crate::gui::{
    atoms::knob,
    parameter::{Parameter, Range},
    parameters::input_gain::InputGain,
    widget::Widget,
};

pub struct InputGainKnob;

impl Widget for InputGainKnob {
    fn element_id(&self) -> &'static str {
        "input-gain"
    }

    fn param_id(&self) -> usize {
        Parameter::<InputGain, Range>::ID
    }

    fn normalize(&self, raw: f32) -> f64 {
        Parameter::<InputGain, Range>::new().normalize(raw as f64)
    }

    fn draw(&self, scene: &mut Scene, x: f64, y: f64, width: f64, height: f64, normalized: f64) {
        let cx = x + width / 2.0;
        let cy = y + height / 2.0;
        let r = width.min(height) / 2.0 - 4.0;
        knob::draw(scene, cx, cy, r, normalized);
    }
}
