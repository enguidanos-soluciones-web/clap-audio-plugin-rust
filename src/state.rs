use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;

use crate::{gui::parameters::any::PARAMS_COUNT, nam};

pub struct GUIState {
    nam_model_rate: AtomicU64, // f64::to_bits(); NaN = not initialized
}

impl GUIState {
    pub fn nam_model_rate(&self) -> Option<f64> {
        let rate = f64::from_bits(self.nam_model_rate.load(Ordering::Relaxed));
        if rate.is_nan() { None } else { Some(rate) }
    }

    pub fn set_nam_model_rate(&self, rate: f64) {
        self.nam_model_rate.store(rate.to_bits(), Ordering::Relaxed);
    }
}

impl Default for GUIState {
    fn default() -> Self {
        Self {
            nam_model_rate: AtomicU64::new(f64::NAN.to_bits()),
        }
    }
}

pub struct PluginState {
    pub gui_window: Option<baseview::WindowHandle>,
    pub gui_width: u32,
    pub gui_height: u32,
    pub gui_state: Arc<GUIState>,

    pub conversion_input_buf: Vec<f64>,
    pub conversion_output_buf: Vec<f64>,

    pub nam_model: Option<cxx::UniquePtr<nam::ffi::NamDsp>>,

    pub parameters_rx: Arc<ArcSwap<PluginParameters>>,
    pub parameters_wx: Arc<Mutex<PluginParameters>>,
}

impl Default for PluginState {
    fn default() -> Self {
        Self {
            gui_window: None,
            gui_width: 800,
            gui_height: 600,
            gui_state: Default::default(),

            conversion_input_buf: Vec::new(),
            conversion_output_buf: Vec::new(),

            nam_model: None,

            parameters_rx: Default::default(),
            parameters_wx: Default::default(),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct PluginParameters {
    pub audio_thread_parameters: [f32; PARAMS_COUNT],
    pub main_thread_parameters: [f32; PARAMS_COUNT],
    pub audio_thread_parameters_changed: [bool; PARAMS_COUNT],
    pub main_thread_parameters_changed: [bool; PARAMS_COUNT],
}
