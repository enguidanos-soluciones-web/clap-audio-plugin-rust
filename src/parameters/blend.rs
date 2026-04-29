use super::{
    PARAMETER_GESTURE_DOUBLE_CLICK, PARAMETER_GESTURE_DRAG, Parameter, ParameterClickable, ParameterDraggable, ProposedParamChange, Range,
};
use crate::gui::colors;
use crate::gui::helpers::{arc_path, full_circle_path};
use crate::gui::widget::Widget;
use std::f64::consts::PI;
use vello::{
    Scene,
    kurbo::{Affine, Circle, Line, Point, Stroke},
    peniko::Fill,
};

#[derive(Clone, Copy)]
pub struct Blend;

impl Parameter<Blend, Range> {
    pub const ID: usize = 3;

    pub fn new() -> Self {
        Self {
            id: Self::ID,
            name: "Blend",
            gestures: PARAMETER_GESTURE_DRAG | PARAMETER_GESTURE_DOUBLE_CLICK,
            behave: Range { min: 0., max: 1., def: 1. },
            _marker_type: std::marker::PhantomData,
            _marker_behaviour: std::marker::PhantomData,
        }
    }

    pub fn as_draggable(&self) -> Option<ParameterDraggable<'_, Blend, Range>> {
        if self.gestures & PARAMETER_GESTURE_DRAG != 0 {
            Some(ParameterDraggable::<Blend, Range>::new(self))
        } else {
            None
        }
    }

    pub fn as_clickable(&self) -> Option<ParameterClickable<'_, Blend, Range>> {
        if self.gestures & PARAMETER_GESTURE_DOUBLE_CLICK != 0 {
            Some(ParameterClickable::<Blend, Range>::new(self))
        } else {
            None
        }
    }

    /// Linearly blends `dry` and `wet` signals according to the normalised parameter `value`.
    ///
    /// - `dry`   — input signal through the Klon buffer stage (high-pass ~15 Hz + FET compression): no input gain, no NAM, no loudness correction.
    /// - `wet`   — NAM-processed signal after DC filter, loudness correction and output gain.
    /// - `value` — normalised blend in `[0.0, 1.0]`: `0.0` = 100 % dry, `1.0` = 100 % wet.
    ///
    /// The tone lowpass filter is applied to the blended result (not per-path) so it shapes
    /// both dry and wet equally with a single stateful filter instance.
    pub fn mix(dry: f64, wet: f64, value: f64) -> f64 {
        value * wet + (1.0 - value) * dry
    }
}

impl<'a> ParameterDraggable<'a, Blend, Range> {
    pub fn new(inner: &'a Parameter<Blend, Range>) -> Self {
        Self {
            inner,
            _marker_type: std::marker::PhantomData,
            _marker_behaviour: std::marker::PhantomData,
        }
    }

    pub fn on_drag(&self, start_pos: (f64, f64), start_value: f64, current_pos: (f64, f64)) -> Option<ProposedParamChange> {
        const SENSITIVITY: f64 = 200.0;

        let delta = (start_pos.1 - current_pos.1) / SENSITIVITY;
        let normalized = (start_value + delta).clamp(0.0, 1.0);
        let value = self.inner.behave.min + normalized * (self.inner.behave.max - self.inner.behave.min);

        Some(ProposedParamChange {
            index: self.inner.id,
            value,
        })
    }
}

impl<'a> ParameterClickable<'a, Blend, Range> {
    pub fn new(inner: &'a Parameter<Blend, Range>) -> Self {
        Self {
            inner,
            _marker_type: std::marker::PhantomData,
            _marker_behaviour: std::marker::PhantomData,
        }
    }

    pub fn on_double_click(&self) -> Option<ProposedParamChange> {
        Some(ProposedParamChange {
            index: self.inner.id,
            value: self.inner.behave.def,
        })
    }
}

impl Widget for Parameter<Blend, Range> {
    fn element_id(&self) -> &'static str {
        "blend"
    }

    fn param_id(&self) -> usize {
        Self::ID
    }

    fn normalize(&self, raw: f64) -> f64 {
        Self::new().normalize(raw)
    }

    fn draw(&self, scene: &mut Scene, x: f64, y: f64, width: f64, height: f64, normalized: f64) {
        const KNOB_START: f64 = 3.0 * PI / 4.0;
        const KNOB_SWEEP: f64 = 3.0 * PI / 2.0;

        let cx = x + width / 2.0;
        let cy = y + height / 2.0;
        let r = width.min(height) / 2.0 - 4.0;

        let center = Point::new(cx, cy);

        scene.fill(Fill::NonZero, Affine::IDENTITY, colors::neutral_600, None, &Circle::new(center, r));

        scene.stroke(
            &Stroke::new(2.0),
            Affine::IDENTITY,
            colors::neutral_800,
            None,
            &arc_path(cx, cy, r - 7.0, KNOB_START, KNOB_SWEEP),
        );

        if normalized > 0.001 {
            scene.stroke(
                &Stroke::new(2.0),
                Affine::IDENTITY,
                colors::amber_500,
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
            colors::white,
            None,
            &Line::new(center, Point::new(ix, iy)),
        );

        scene.stroke(
            &Stroke::new(1.0),
            Affine::IDENTITY,
            colors::neutral_900,
            None,
            &full_circle_path(cx, cy, r),
        );
    }
}
