use std::f64::consts::PI;
use vello::{
    Scene,
    kurbo::{Affine, Circle, Line, Point, Stroke},
    peniko::{Color, Fill},
};

use crate::gui::helpers::{arc_path, full_circle_path};

pub const KNOB_START: f64 = 3.0 * PI / 4.0;
pub const KNOB_SWEEP: f64 = 3.0 * PI / 2.0;

pub fn draw(scene: &mut Scene, cx: f64, cy: f64, r: f64, normalized: f64) {
    let center = Point::new(cx, cy);

    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgba8(58, 58, 63, 255),
        None,
        &Circle::new(center, r),
    );

    scene.stroke(
        &Stroke::new(2.0),
        Affine::IDENTITY,
        Color::from_rgba8(42, 42, 46, 255),
        None,
        &arc_path(cx, cy, r - 7.0, KNOB_START, KNOB_SWEEP),
    );

    if normalized > 0.001 {
        scene.stroke(
            &Stroke::new(2.0),
            Affine::IDENTITY,
            Color::from_rgba8(190, 100, 40, 255),
            None,
            &arc_path(cx, cy, r - 7.0, KNOB_START, normalized * KNOB_SWEEP),
        );
    }

    let angle = KNOB_START + normalized * KNOB_SWEEP;
    let ix = cx + (r - 12.0) * angle.cos();
    let iy = cy + (r - 12.0) * angle.sin();
    scene.stroke(
        &Stroke::new(1.5),
        Affine::IDENTITY,
        Color::from_rgba8(240, 240, 240, 255),
        None,
        &Line::new(center, Point::new(ix, iy)),
    );

    scene.stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        Color::from_rgba8(85, 85, 90, 255),
        None,
        &full_circle_path(cx, cy, r),
    );
}
