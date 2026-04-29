use vello::Scene;

pub trait Widget {
    fn element_id(&self) -> &'static str;

    fn param_id(&self) -> usize;

    fn normalize(&self, raw: f64) -> f64;

    /// Paint the widget into `scene`.
    ///
    /// - `x`, `y`      — top-left corner of the bounding rect in window pixels (from the DOM layout).
    /// - `width`, `height` — size of that rect in pixels.
    /// - `normalized`  — parameter value mapped to `[0.0, 1.0]` via [`Widget::normalize`].
    ///                   Implementations use this to position indicators (e.g. knob angle, fader pos).
    fn draw(&self, scene: &mut Scene, x: f64, y: f64, width: f64, height: f64, normalized: f64);
}
