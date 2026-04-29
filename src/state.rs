use crate::channel::{Receiver, Sender};
use crate::clap::*;
use crate::dsp::dc_filter::DcFilter;
use crate::{dsp::nam, parameters::any::PARAMS_COUNT};
use arc_swap::ArcSwap;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug)]
pub enum ParamEvent {
    Ack,
    Nack { id: usize },
    Automation { id: usize, value: f32 },
}

#[derive(Debug)]
pub struct ParamChange {
    pub id: usize,
    pub value: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamSnapshot {
    pub values: [f32; PARAMS_COUNT],
}

pub struct AudioThreadState {
    pub host: *const clap_host_t,
    pub sample_rate: f64,

    pub nam_model: Option<cxx::UniquePtr<nam::ffi::NamDsp>>,
    pub nam_loudness_correction: f64,

    pub input_buf: Vec<f64>,
    pub output_buf: Vec<f64>,

    pub dc_filter: DcFilter,

    pub daw_events: Sender<ParamEvent>,
    pub param_changes: Receiver<ParamChange>,
    pub param_snapshot: Arc<ArcSwap<ParamSnapshot>>,

    pub thread_id: Option<std::thread::ThreadId>,
}

impl AudioThreadState {
    pub fn assert_audio_thread(&self) {
        debug_assert_eq!(
            std::thread::current().id(),
            self.thread_id.expect("premature access to audio thread id"),
            "AudioThreadState accessed from wrong thread!"
        );
    }
}

pub struct MainThreadState {
    pub param_snapshot: Arc<ArcSwap<ParamSnapshot>>,

    pub daw_events: Receiver<ParamEvent>,
    pub gui_changes: Receiver<ParamChange>,
    pub param_changes: Sender<ParamChange>,

    pub gui_shared: Arc<ArcSwap<GUIShared>>,
    pub gui_window: Option<baseview::WindowHandle>,
    pub gui_width: u32,
    pub gui_height: u32,

    pub thread_id: Option<std::thread::ThreadId>,
}

impl MainThreadState {
    pub fn assert_main_thread(&self) {
        debug_assert_eq!(
            std::thread::current().id(),
            self.thread_id.expect("premature access to main thread"),
            "MainThreadState accessed from wrong thread!"
        );
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GUIShared {
    pub nam_model_rate: Option<u64>,
}
