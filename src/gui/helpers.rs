use std::f64::consts::PI;
use vello::kurbo::{BezPath, Point};

pub fn arc_path(cx: f64, cy: f64, r: f64, start: f64, sweep: f64) -> BezPath {
    const STEPS: usize = 48;
    let mut path = BezPath::new();
    for i in 0..=STEPS {
        let a = start + (i as f64 / STEPS as f64) * sweep;
        let x = cx + r * a.cos();
        let y = cy + r * a.sin();
        if i == 0 {
            path.move_to(Point::new(x, y));
        } else {
            path.line_to(Point::new(x, y));
        }
    }
    path
}

pub fn full_circle_path(cx: f64, cy: f64, r: f64) -> BezPath {
    arc_path(cx, cy, r, 0.0, 2.0 * PI)
}
