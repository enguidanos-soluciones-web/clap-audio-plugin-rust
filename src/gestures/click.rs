use crate::parameters::{
    PARAMETER_GESTURE_DOUBLE_CLICK, ProposedParamChange, any::AnyParameter,
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
        };

        if gestures & PARAMETER_GESTURE_DOUBLE_CLICK == 0 {
            return None;
        }

        Some(ActiveClick(param))
    }

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
