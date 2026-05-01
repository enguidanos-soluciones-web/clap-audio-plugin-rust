use crate::parameters::any::PARAMS_COUNT;

#[derive(Clone, Default)]
pub struct AppState {
    pub params: [f64; PARAMS_COUNT],
    pub model_name: Option<String>,
    pub model_rate: Option<f64>,
}
