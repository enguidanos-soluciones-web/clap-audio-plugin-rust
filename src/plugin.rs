use crate::{
    clap::*,
    descriptor::PLUGIN_DESCRIPTOR,
    extensions::{audio_ports::AUDIO_PORTS_EXT, parameters::PARAMETERS_EXT, state::STATE_EXT},
    parameters::any::PARAMS_COUNT,
    nam, plugin,
    processors::{handle_clap_event::handle_clap_event, render_audio::render_audio, sync_main_to_audio::sync_main_to_audio},
};
use arc_swap::ArcSwap;
use std::{
    ffi::{CStr, c_char, c_void},
    sync::{Arc, Mutex},
};

const MODEL_JSON: &str = include_str!("../models/amp_drive.nam");

#[derive(Default, Clone, Copy)]
pub struct PluginParameters {
    pub audio_thread_parameters: [f32; PARAMS_COUNT],
    pub main_thread_parameters: [f32; PARAMS_COUNT],
    pub audio_thread_parameters_changed: [bool; PARAMS_COUNT],
    pub main_thread_parameters_changed: [bool; PARAMS_COUNT],
}

pub struct Plugin {
    pub inner: clap_plugin_t,
    pub host: *const clap_host_t,
    pub sample_rate: f64,
    pub model: Option<cxx::UniquePtr<nam::ffi::NamDsp>>,
    pub input_buf: Vec<f64>,
    pub output_buf: Vec<f64>,
    pub parameters_rx: Arc<ArcSwap<PluginParameters>>,
    pub parameters_wx: Arc<Mutex<PluginParameters>>,
}

pub const PLUGIN_CLASS: clap_plugin_t = clap_plugin_t {
    desc: &PLUGIN_DESCRIPTOR,
    plugin_data: std::ptr::null_mut(),
    init: Some(plugin::init),
    destroy: Some(plugin::destroy),
    activate: Some(plugin::activate),
    deactivate: Some(plugin::deactivate),
    start_processing: Some(plugin::start_processing),
    stop_processing: Some(plugin::stop_processing),
    reset: Some(plugin::reset),
    process: Some(plugin::process),
    get_extension: Some(plugin::get_extension),
    on_main_thread: Some(plugin::on_main_thread),
};

pub unsafe extern "C" fn init(plugin: *const clap_plugin_t) -> bool {
    nam::ffi::activation_enable_fast_tanh();

    let plugin_data = unsafe { (*plugin).plugin_data as *const Plugin };
    let plugin_ref = unsafe { plugin_data.as_ref_unchecked() };

    let mut anychanged = false;

    if let Ok(mut params) = plugin_ref.parameters_wx.try_lock() {
        for n in 0..PARAMS_COUNT {
            let mut information = unsafe { std::mem::zeroed::<clap_param_info_t>() };

            if let Some(get_info) = PARAMETERS_EXT.get_info {
                if unsafe { get_info(plugin, n as u32, &mut information as *mut clap_param_info_t) } {
                    params.main_thread_parameters[n] = information.default_value as f32;
                    params.audio_thread_parameters[n] = information.default_value as f32;
                    anychanged = true;
                }
            }
        }

        if anychanged {
            plugin_ref.parameters_rx.store(Arc::new(*params));
        }
    }

    anychanged
}

pub unsafe extern "C" fn destroy(plugin: *const clap_plugin_t) {
    let plugin = unsafe { (*plugin).plugin_data as *mut Plugin };
    drop(unsafe { Box::from_raw(plugin) });
}

pub unsafe extern "C" fn activate(plugin: *const clap_plugin, sample_rate: f64, _min_frames_count: u32, max_frames_count: u32) -> bool {
    let plugin = unsafe { (*plugin).plugin_data as *mut Plugin };
    let plugin_ref = unsafe { plugin.as_mut_unchecked() };

    plugin_ref.sample_rate = sample_rate;
    plugin_ref.input_buf = vec![0.0f64; max_frames_count as usize];
    plugin_ref.output_buf = vec![0.0f64; max_frames_count as usize];

    let mut model = nam::ffi::dsp_load(MODEL_JSON);
    nam::ffi::dsp_reset(model.pin_mut(), sample_rate, max_frames_count as i32);
    plugin_ref.model = Some(model);

    true
}

pub unsafe extern "C" fn deactivate(_plugin: *const clap_plugin) {}

pub unsafe extern "C" fn start_processing(_plugin: *const clap_plugin) -> bool {
    true
}

pub unsafe extern "C" fn stop_processing(_plugin: *const clap_plugin) {}

pub unsafe extern "C" fn reset(plugin: *const clap_plugin) {
    let plugin = unsafe { (*plugin).plugin_data as *mut Plugin };
    let plugin_ref = unsafe { plugin.as_mut_unchecked() };

    if let Some(model) = plugin_ref.model.as_mut() {
        nam::ffi::dsp_reset(model.pin_mut(), plugin_ref.sample_rate, plugin_ref.input_buf.len() as i32);
    }
}

pub unsafe extern "C" fn get_extension(_plugin: *const clap_plugin, id: *const c_char) -> *const c_void {
    if unsafe { CStr::from_ptr(id) } == CLAP_EXT_AUDIO_PORTS {
        return &AUDIO_PORTS_EXT as *const _ as *const c_void;
    }
    if unsafe { CStr::from_ptr(id) } == CLAP_EXT_PARAMS {
        return &PARAMETERS_EXT as *const _ as *const c_void;
    }
    if unsafe { CStr::from_ptr(id) } == CLAP_EXT_STATE {
        return &STATE_EXT as *const _ as *const c_void;
    }
    std::ptr::null()
}

pub unsafe extern "C" fn on_main_thread(_plugin: *const clap_plugin) {}

pub unsafe extern "C" fn process(plugin: *const clap_plugin, process: *const clap_process_t) -> clap_process_status {
    let plugin = unsafe { (*plugin).plugin_data as *mut Plugin };
    let plugin_ref = unsafe { plugin.as_mut_unchecked() };
    let process_ref = unsafe { process.as_ref_unchecked() };

    unsafe { sync_main_to_audio(plugin_ref, process_ref.out_events.cast_mut()) };

    let in_events = unsafe { process_ref.in_events.as_ref_unchecked() };

    let event_count = in_events.size.map(|f| unsafe { f(process_ref.in_events) }).unwrap_or_default();

    for i in 0..event_count {
        if let Some(get) = in_events.get {
            let event = unsafe { get(process_ref.in_events, i) };
            unsafe { handle_clap_event(plugin_ref, event) };
        }
    }

    debug_assert!(process_ref.audio_inputs_count == 1);
    debug_assert!(process_ref.audio_outputs_count == 1);

    let audio_input = unsafe { *process_ref.audio_inputs.as_ref_unchecked().data32.offset(0) };
    let audio_output = unsafe { *process_ref.audio_outputs.as_mut_unchecked().data32.offset(0) };

    unsafe { render_audio(plugin_ref, audio_input, audio_output, process_ref.frames_count as usize) };

    CLAP_PROCESS_CONTINUE as clap_process_status
}
