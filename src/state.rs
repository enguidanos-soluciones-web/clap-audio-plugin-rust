use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;

use crate::{gui::parameters::any::PARAMS_COUNT, nam, processors::handle_gui_event::GUIEvent};

pub struct PluginState {
    pub gui_window: Option<baseview::WindowHandle>,
    pub gui_width: u32,
    pub gui_height: u32,
    pub gui_queue: Arc<Mutex<Vec<GUIEvent>>>,

    pub conversion_input_buf: Vec<f64>,
    pub conversion_output_buf: Vec<f64>,

    pub nam_model_sample_rate: f64,
    pub nam_model: Option<cxx::UniquePtr<nam::ffi::NamDsp>>,

    pub parameters_rx: Arc<ArcSwap<PluginParameters>>,
    pub parameters_wx: Arc<Mutex<PluginParameters>>,
}

impl Default for PluginState {
    fn default() -> Self {
        Self {
            gui_window: None,
            gui_width: 600,
            gui_height: 400,
            gui_queue: Default::default(),

            conversion_input_buf: Vec::new(),
            conversion_output_buf: Vec::new(),

            nam_model: None,
            nam_model_sample_rate: 0.0,

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
