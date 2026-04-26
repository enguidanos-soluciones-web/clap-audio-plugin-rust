use crate::parameters::base::{Parameter, Range};

#[derive(Clone, Copy)]
pub struct InputGain;

impl Parameter<InputGain, Range> {
    pub const ID: usize = 0;

    pub fn new() -> Self {
        Self {
            name: "Input Gain",
            behave: Range {
                min: -20.0,
                max: 20.0,
                def: 0.0,
            },
            _marker: std::marker::PhantomData,
        }
    }
}
