use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub struct Range {
    pub min: f64,
    pub max: f64,
    pub def: f64,
}

#[derive(Clone, Copy)]
pub struct Parameter<T, R> {
    pub name: &'static str,
    pub behave: R,
    pub _marker: PhantomData<T>,
}
