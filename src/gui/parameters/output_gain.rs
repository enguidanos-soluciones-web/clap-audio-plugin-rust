use crate::gui::parameter::{
    PARAMETER_GESTURE_CLICK, PARAMETER_GESTURE_DRAG, Parameter, ParameterClickable, ParameterDraggable, ProposedParamChange, Range,
};

#[derive(Clone, Copy)]
pub struct OutputGain;

pub const OUTPUT_GAIN_ID: usize = Parameter::<OutputGain, Range>::ID;

impl Parameter<OutputGain, Range> {
    pub const ID: usize = 1;

    pub fn new() -> Self {
        Self {
            id: Self::ID,
            name: "Output Gain",
            gestures: PARAMETER_GESTURE_DRAG | PARAMETER_GESTURE_CLICK,
            behave: Range {
                min: -40.0,
                max: 40.0,
                def: 0.0,
            },
            _marker_type: std::marker::PhantomData,
            _marker_behaviour: std::marker::PhantomData,
        }
    }

    pub fn as_draggable(&self) -> Option<ParameterDraggable<'_, OutputGain, Range>> {
        if self.gestures & PARAMETER_GESTURE_DRAG != 0 {
            Some(ParameterDraggable::<OutputGain, Range>::new(self))
        } else {
            None
        }
    }

    pub fn as_clickable(&self) -> Option<ParameterClickable<'_, OutputGain, Range>> {
        if self.gestures & PARAMETER_GESTURE_CLICK != 0 {
            Some(ParameterClickable::<OutputGain, Range>::new(self))
        } else {
            None
        }
    }
}

impl<'a> ParameterDraggable<'a, OutputGain, Range> {
    pub fn new(inner: &'a Parameter<OutputGain, Range>) -> Self {
        Self {
            inner,
            _marker_type: std::marker::PhantomData,
            _marker_behaviour: std::marker::PhantomData,
        }
    }

    /// Vertical drag: dragging up increases gain, dragging down decreases it.
    ///
    /// Y axis grows downward in screen space, so the delta is `start_y - current_y`:
    /// moving up gives a positive delta (value increases), moving down a negative one.
    ///
    /// `SENSITIVITY` sets drag resolution: that many pixels of travel covers the full
    /// normalized range [0.0, 1.0], i.e. the entire [-40 dB, +40 dB] span.
    pub fn on_drag(&self, start_pos: (f32, f32), start_value: f32, current_pos: (f32, f32)) -> Option<ProposedParamChange> {
        const SENSITIVITY: f32 = 200.0;

        let delta = (start_pos.1 - current_pos.1) / SENSITIVITY;
        let normalized = (start_value + delta).clamp(0.0, 1.0) as f64;
        let value = self.inner.behave.min + normalized * (self.inner.behave.max - self.inner.behave.min);

        Some(ProposedParamChange {
            index: self.inner.id,
            value: value as f32,
        })
    }
}

impl<'a> ParameterClickable<'a, OutputGain, Range> {
    pub fn new(inner: &'a Parameter<OutputGain, Range>) -> Self {
        Self {
            inner,
            _marker_type: std::marker::PhantomData,
            _marker_behaviour: std::marker::PhantomData,
        }
    }

    pub fn on_double_click(&self) -> Option<ProposedParamChange> {
        Some(ProposedParamChange {
            index: self.inner.id,
            value: self.inner.behave.def as f32,
        })
    }
}
