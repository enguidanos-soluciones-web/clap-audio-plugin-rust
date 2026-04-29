use super::{Action, Parameter, Range};
use crate::parameters::{blend::Blend, input_gain::InputGain, load_model::LoadModel, output_gain::OutputGain, tone::Tone};

pub const PARAMS_COUNT: usize = 5;

pub enum AnyParameter {
    InputGain { inner: Parameter<InputGain, Range> },
    OutputGain { inner: Parameter<OutputGain, Range> },
    Tone { inner: Parameter<Tone, Range> },
    Blend { inner: Parameter<Blend, Range> },
    LoadModel { inner: Parameter<LoadModel, Action> },
}

impl TryFrom<usize> for AnyParameter {
    type Error = ();

    fn try_from(id: usize) -> Result<Self, Self::Error> {
        match id {
            Parameter::<InputGain, Range>::ID => Ok(AnyParameter::InputGain {
                inner: Parameter::<InputGain, Range>::new(),
            }),
            Parameter::<OutputGain, Range>::ID => Ok(AnyParameter::OutputGain {
                inner: Parameter::<OutputGain, Range>::new(),
            }),
            Parameter::<Tone, Range>::ID => Ok(AnyParameter::Tone {
                inner: Parameter::<Tone, Range>::new(),
            }),
            Parameter::<Blend, Range>::ID => Ok(AnyParameter::Blend {
                inner: Parameter::<Blend, Range>::new(),
            }),
            Parameter::<LoadModel, Action>::ID => Ok(AnyParameter::LoadModel {
                inner: Parameter::<LoadModel, Action>::new(),
            }),
            _ => Err(()),
        }
    }
}
