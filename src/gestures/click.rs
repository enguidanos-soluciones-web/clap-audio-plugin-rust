use crate::{
    parameters::{
        PARAMETER_GESTURE_DOUBLE_CLICK, PARAMETER_GESTURE_SINGLE_CLICK, ProposedParamChange,
        any::AnyParameter,
    },
    state::GuiRequest,
};

pub struct ActiveClick(AnyParameter);

impl ActiveClick {
    pub fn from_index(index: usize) -> Option<Self> {
        let param = AnyParameter::try_from(index).ok()?;

        let gestures = match &param {
            AnyParameter::InputGain { inner } => inner.gestures,
            AnyParameter::OutputGain { inner } => inner.gestures,
            AnyParameter::Tone { inner } => inner.gestures,
            AnyParameter::Blend { inner } => inner.gestures,
            AnyParameter::LoadModel { inner } => inner.gestures,
        };

        let is_clickable = gestures & (PARAMETER_GESTURE_SINGLE_CLICK | PARAMETER_GESTURE_DOUBLE_CLICK) != 0;

        if !is_clickable {
            return None;
        }

        Some(ActiveClick(param))
    }

    /// Fires on a single click. Only parameters with `PARAMETER_GESTURE_SINGLE_CLICK` respond.
    pub fn on_single_click(&self) -> Option<GuiRequest> {
        match &self.0 {
            AnyParameter::LoadModel { inner } if inner.gestures & PARAMETER_GESTURE_SINGLE_CLICK != 0 => {
                Some(inner.on_single_click())
            }
            _ => None,
        }
    }

    /// Fires on a double click. Only parameters with `PARAMETER_GESTURE_DOUBLE_CLICK` respond.
    pub fn on_double_click(&self) -> Option<ProposedParamChange> {
        match &self.0 {
            AnyParameter::InputGain { inner } if inner.gestures & PARAMETER_GESTURE_DOUBLE_CLICK != 0 => {
                inner.as_clickable()?.on_double_click()
            }
            AnyParameter::OutputGain { inner } if inner.gestures & PARAMETER_GESTURE_DOUBLE_CLICK != 0 => {
                inner.as_clickable()?.on_double_click()
            }
            AnyParameter::Tone { inner } if inner.gestures & PARAMETER_GESTURE_DOUBLE_CLICK != 0 => {
                inner.as_clickable()?.on_double_click()
            }
            AnyParameter::Blend { inner } if inner.gestures & PARAMETER_GESTURE_DOUBLE_CLICK != 0 => {
                inner.as_clickable()?.on_double_click()
            }
            _ => None,
        }
    }
}
