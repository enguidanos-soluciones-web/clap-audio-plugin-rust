use crate::parameters::{
    base::{Parameter, Range},
    input_gain::InputGain,
    output_gain::OutputGain,
};

pub const PARAMS_COUNT: usize = 2;

pub enum AnyParameter {
    InputGain { inner: Parameter<InputGain, Range> },
    OutputGain { inner: Parameter<OutputGain, Range> },
}

impl TryFrom<usize> for AnyParameter {
    type Error = ();

    fn try_from(index: usize) -> Result<Self, Self::Error> {
        match index {
            Parameter::<InputGain, Range>::ID => Ok(AnyParameter::InputGain {
                inner: Parameter::<InputGain, Range>::new(),
            }),
            Parameter::<OutputGain, Range>::ID => Ok(AnyParameter::OutputGain {
                inner: Parameter::<OutputGain, Range>::new(),
            }),
            _ => Err(()),
        }
    }
}
