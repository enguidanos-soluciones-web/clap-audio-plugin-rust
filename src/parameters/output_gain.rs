use crate::parameters::base::{Parameter, Range};

#[derive(Clone, Copy)]
pub struct OutputGain;

impl Parameter<OutputGain, Range> {
    pub const ID: usize = 1;

    pub fn new() -> Self {
        Self {
            name: "Output Gain",
            behave: Range {
                min: -40.0,
                max: 40.0,
                def: 0.0,
            },
            _marker: std::marker::PhantomData,
        }
    }
}
