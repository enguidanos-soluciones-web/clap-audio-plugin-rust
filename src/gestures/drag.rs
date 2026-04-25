use crate::gui::{parameter::ProposedParamChange, parameters::any::AnyParameter};

pub struct ActiveDrag {
    param: AnyParameter,
    start_pos: (f32, f32),
    start_value: f32,
}

impl ActiveDrag {
    pub fn from_index(index: usize, x: f32, y: f32, raw: f32) -> Option<Self> {
        let param = AnyParameter::try_from(index).ok()?;

        let start_value = match &param {
            AnyParameter::InputGain { inner } => inner.normalize(raw as f64) as f32,
            AnyParameter::OutputGain { inner } => inner.normalize(raw as f64) as f32,
        };

        let is_draggable = match &param {
            AnyParameter::InputGain { inner } => inner.as_draggable().is_some(),
            AnyParameter::OutputGain { inner } => inner.as_draggable().is_some(),
        };

        if !is_draggable {
            return None;
        }

        Some(Self {
            param,
            start_pos: (x, y),
            start_value,
        })
    }

    pub fn on_drag(&self, x: f32, y: f32) -> Option<ProposedParamChange> {
        match &self.param {
            AnyParameter::InputGain { inner } => inner.as_draggable()?.on_drag(self.start_pos, self.start_value, (x, y)),
            AnyParameter::OutputGain { inner } => inner.as_draggable()?.on_drag(self.start_pos, self.start_value, (x, y)),
        }
    }
}
