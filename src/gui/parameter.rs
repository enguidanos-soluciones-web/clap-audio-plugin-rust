use std::marker::PhantomData;

pub const PARAMETER_GESTURE_DRAG: u8 = 1 << 0;
pub const PARAMETER_GESTURE_CLICK: u8 = 1 << 1;

pub struct ProposedParamChange {
    pub index: usize,
    pub value: f32,
}

#[derive(Clone, Copy)]
pub struct Range {
    pub min: f64,
    pub max: f64,
    pub def: f64,
}

#[derive(Clone, Copy)]
pub struct Parameter<T, R> {
    pub id: usize,
    pub name: &'static str,
    pub gestures: u8,
    pub behave: R,
    pub _marker_type: PhantomData<T>,
    pub _marker_behaviour: PhantomData<R>,
}

#[derive(Clone, Copy)]
pub struct ParameterDraggable<'a, T, R> {
    pub inner: &'a Parameter<T, R>,
    pub _marker_type: PhantomData<T>,
    pub _marker_behaviour: PhantomData<R>,
}

#[derive(Clone, Copy)]
pub struct ParameterClickable<'a, T, R> {
    pub inner: &'a Parameter<T, R>,
    pub _marker_type: PhantomData<T>,
    pub _marker_behaviour: PhantomData<R>,
}

impl<T> Parameter<T, Range> {
    pub fn normalize(&self, value: f64) -> f64 {
        ((value - self.behave.min) / (self.behave.max - self.behave.min)).clamp(0.0, 1.0)
    }
}
