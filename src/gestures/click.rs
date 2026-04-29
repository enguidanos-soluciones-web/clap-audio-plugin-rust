use crate::parameters::{ProposedParamChange, any::AnyParameter};

pub struct ActiveClick(AnyParameter);

impl ActiveClick {
    pub fn from_index(index: usize) -> Option<Self> {
        let param = AnyParameter::try_from(index).ok()?;

        let is_clickable = match &param {
            AnyParameter::InputGain { inner } => inner.as_clickable().is_some(),
            AnyParameter::OutputGain { inner } => inner.as_clickable().is_some(),
            AnyParameter::Tone { inner } => inner.as_clickable().is_some(),
            AnyParameter::Blend { inner } => inner.as_clickable().is_some(),
        };

        if !is_clickable {
            return None;
        }

        Some(ActiveClick(param))
    }

    pub fn on_double_click(&self) -> Option<ProposedParamChange> {
        match &self.0 {
            AnyParameter::InputGain { inner } => inner.as_clickable()?.on_double_click(),
            AnyParameter::OutputGain { inner } => inner.as_clickable()?.on_double_click(),
            AnyParameter::Tone { inner } => inner.as_clickable()?.on_double_click(),
            AnyParameter::Blend { inner } => inner.as_clickable()?.on_double_click(),
        }
    }
}
