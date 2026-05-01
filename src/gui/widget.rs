use vello::Scene;

use crate::gui::text::TextRenderer;

pub trait Widget {
    fn dom_id(&self) -> &'static str;

    fn param_id(&self) -> usize;

    fn draw(
        &self,
        scene: &mut Scene,
        text: &mut TextRenderer,
        coordinates: (f64, f64),
        dimensions: (f64, f64),
        cursor: (f64, f64),
        value: f64,
    );
}
