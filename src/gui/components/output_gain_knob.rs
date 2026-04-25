use vello::Scene;

use crate::gui::{
    atoms::knob,
    parameter::{Parameter, Range},
    parameters::output_gain::OutputGain,
    widget::Widget,
};

pub struct OutputGainKnob;

impl Widget for OutputGainKnob {
    fn element_id(&self) -> &'static str {
        "output-gain"
    }

    fn param_id(&self) -> usize {
        Parameter::<OutputGain, Range>::ID
    }

    fn normalize(&self, raw: f32) -> f64 {
        Parameter::<OutputGain, Range>::new().normalize(raw as f64)
    }

    fn draw(&self, scene: &mut Scene, x: f64, y: f64, width: f64, height: f64, normalized: f64) {
        let cx = x + width / 2.0;
        let cy = y + height / 2.0;
        let r = width.min(height) / 2.0 - 4.0;
        knob::draw(scene, cx, cy, r, normalized);
    }
}
