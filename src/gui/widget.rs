use vello::Scene;

pub trait Widget {
    fn element_id(&self) -> &'static str;

    fn param_id(&self) -> usize;

    /// Maps a raw parameter value to `[0.0, 1.0]`. Defaults to identity — override for
    /// widgets whose raw range differs from `[0.0, 1.0]`.
    fn normalize(&self, raw: f64) -> f64 {
        raw
    }

    /// Paint the widget into `scene`.
    ///
    /// - `x`, `y`      — top-left corner of the bounding rect in window pixels (from the DOM layout).
    /// - `width`, `height` — size of that rect in pixels.
    /// - `normalized`  — parameter value mapped to `[0.0, 1.0]` via [`Widget::normalize`].
    ///                   Implementations use this to position indicators (e.g. knob angle, fader pos).
    ///
    /// Defaults to a no-op — override for widgets that need Vello rendering.
    fn draw(&self, _scene: &mut Scene, _x: f64, _y: f64, _width: f64, _height: f64, _normalized: f64) {}
}
