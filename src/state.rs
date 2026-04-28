use crate::clap::*;
use crate::dsp::dc_filter::DcFilter;
use crate::{dsp::nam, parameters::any::PARAMS_COUNT};
use arc_swap::ArcSwap;
use rtrb::{Consumer, Producer};
use std::sync::Arc;

pub enum ParamEvent {
    Ack,
    Nack { id: usize },
    Automation { id: usize, value: f32 },
}

pub struct ParamChange {
    pub id: usize,
    pub value: f32,
}

#[derive(Clone, Copy)]
pub struct ParamSnapshot {
    pub values: [f32; PARAMS_COUNT],
}

pub struct AudioThreadState {
    pub host: *const clap_host_t,
    pub sample_rate: f64,

    pub nam_model: Option<cxx::UniquePtr<nam::ffi::NamDsp>>,

    pub input_buf: Vec<f64>,
    pub output_buf: Vec<f64>,

    pub dc_filter: DcFilter,

    pub param_changes_rx: Consumer<ParamChange>,
    pub param_snapshot: Arc<ArcSwap<ParamSnapshot>>,

    pub daw_events_tx: Producer<ParamEvent>,

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
    pub param_changes_tx: Producer<ParamChange>,
    pub param_snapshot: Arc<ArcSwap<ParamSnapshot>>,

    pub daw_events_rx: Consumer<ParamEvent>,

    pub ui_changes_rx: Consumer<ParamChange>,

    // MainThread State is created under plugin.init(). We dont have yet initialized the AudioThread.
    // We need to save temporally the producers/consumers of the AudioThread.
    pub pending_param_changes_rx: Option<Consumer<ParamChange>>,
    pub pending_daw_events_tx: Option<Producer<ParamEvent>>,
    pub pending_gui_changes_tx: Option<Producer<ParamChange>>,

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

#[derive(Default, Clone, Copy)]
pub struct GUIShared {
    pub nam_model_rate: Option<u64>,
}
