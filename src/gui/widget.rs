use vello::Scene;

pub trait Widget {
    fn element_id(&self) -> &'static str;

    fn param_id(&self) -> usize;

    fn normalize(&self, raw: f32) -> f64;

    fn draw(&self, scene: &mut Scene, x: f64, y: f64, width: f64, height: f64, normalized: f64);
}
