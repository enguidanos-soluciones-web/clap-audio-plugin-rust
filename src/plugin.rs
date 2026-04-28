use crate::{
    clap::*,
    descriptor::PLUGIN_DESCRIPTOR,
    dsp::{self, dc_filter::DcFilter},
    extensions::{audio_ports::AUDIO_PORTS_EXT, gui::GUI_EXT, parameters::PARAMETERS_EXT, state::STATE_EXT},
    parameters::any::PARAMS_COUNT,
    plugin,
    processors::{handle_clap_event::handle_clap_event, render_audio::render_audio, sync_main_to_audio::sync_main_to_audio},
    state::{AudioThreadState, MainThreadState, ParamChange, ParamEvent, ParamSnapshot},
};
use arc_swap::ArcSwap;
use rtrb::RingBuffer;
use std::{
    ffi::{CStr, c_char, c_void},
    sync::Arc,
};

const MODEL_JSON: &str = include_str!("../models/amp_drive.nam");

pub struct Plugin {
    pub inner: clap_plugin_t,
    pub host: *const clap_host_t,
    pub main_thread: Option<MainThreadState>,   // None until init()
    pub audio_thread: Option<AudioThreadState>, // None until activate()
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

// [main-thread]
pub unsafe extern "C" fn init(plugin: *const clap_plugin_t) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };

    let (daw_events_tx, daw_events_rx) = RingBuffer::new(64);
    let (gui_changes_tx, gui_changes_rx) = RingBuffer::new(64);
    let (param_changes_tx, param_changes_rx) = RingBuffer::new(64);

    plugin_ref.main_thread = Some(MainThreadState {
        param_changes_tx,
        param_snapshot: Arc::new(ArcSwap::new(Arc::new(ParamSnapshot {
            values: [0.0f32; PARAMS_COUNT],
        }))),

        daw_events_rx,

        ui_changes_rx: gui_changes_rx,

        pending_param_changes_rx: Some(param_changes_rx),
        pending_daw_events_tx: Some(daw_events_tx),
        pending_gui_changes_tx: Some(gui_changes_tx),

        gui_shared: Default::default(),
        gui_window: None,
        gui_width: 800,
        gui_height: 400,

        thread_id: Some(std::thread::current().id()),
    });

    let main_thread = plugin_ref.main_thread.as_mut().unwrap();
    let mut default_values = [0.0f32; PARAMS_COUNT];
    for n in 0..PARAMS_COUNT {
        let mut information = unsafe { std::mem::zeroed::<clap_param_info_t>() };
        if let Some(get_info) = PARAMETERS_EXT.get_info {
            // SAFETY: MAIN-THREAD must have FIRST THREAD_ID as SOME. OTHERWISE get_info WILL PANIC.
            if unsafe { get_info(plugin, n as u32, &mut information) } {
                default_values[n] = information.default_value as f32;
            }
        }
    }

    main_thread.param_snapshot.store(Arc::new(ParamSnapshot { values: default_values }));

    true
}

pub unsafe extern "C" fn destroy(plugin: *const clap_plugin_t) {
    let plugin = unsafe { (*plugin).plugin_data as *mut Plugin };
    drop(unsafe { Box::from_raw(plugin) });
}

// [main-thread & !active]
pub unsafe extern "C" fn activate(plugin: *const clap_plugin, sample_rate: f64, _min_frames_count: u32, max_frames_count: u32) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };

    if plugin_ref.audio_thread.is_some() {
        #[cfg(debug_assertions)]
        println!("Audio Thread already activated");

        return true;
    }

    let main_thread = plugin_ref.main_thread.as_mut().expect("main thread not initialized");
    main_thread.assert_main_thread();

    dsp::nam::ffi::activation_enable_fast_tanh();
    let mut model = dsp::nam::ffi::dsp_load(MODEL_JSON);
    dsp::nam::ffi::dsp_reset(model.pin_mut(), sample_rate, max_frames_count as i32);

    let mut new_gui_shared = *main_thread.gui_shared.load_full();
    new_gui_shared.nam_model_rate = Some(dsp::nam::ffi::get_sample_rate_from_nam_file(MODEL_JSON) as u64);
    main_thread.gui_shared.store(Arc::new(new_gui_shared));

    plugin_ref.audio_thread = Some(AudioThreadState {
        host: plugin_ref.host,
        sample_rate,

        nam_model: Some(model),

        input_buf: vec![0.0f64; max_frames_count as usize],
        output_buf: vec![0.0f64; max_frames_count as usize],

        dc_filter: DcFilter::new(20.0, sample_rate),

        param_snapshot: Arc::clone(&main_thread.param_snapshot),
        param_changes_rx: main_thread
            .pending_param_changes_rx
            .take()
            .expect("main thread pending_param_changes_rx not initialized"),

        daw_events_tx: main_thread
            .pending_daw_events_tx
            .take()
            .expect("main thread pending_daw_events_tx not initialized"),

        thread_id: None,
    });

    true
}

// [main-thread & active]
pub unsafe extern "C" fn deactivate(plugin: *const clap_plugin) {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };

    if plugin_ref.audio_thread.is_none() {
        #[cfg(debug_assertions)]
        println!("Audio Thread not yet active");

        return;
    }

    let main_thread = plugin_ref.main_thread.as_mut().expect("main thread not initialized");
    main_thread.assert_main_thread();

    if let Some(audio_thread) = plugin_ref.audio_thread.take() {
        // Drain pending events from audio before drop because we don't want to
        // loose the automatization that arrived just before 'deactivate'.
        while let Ok(event) = main_thread.daw_events_rx.pop() {
            match event {
                ParamEvent::Automation { id, value } => {
                    let mut new_snapshot = *main_thread.param_snapshot.load_full();
                    new_snapshot.values[id] = value;
                    main_thread.param_snapshot.store(Arc::new(new_snapshot));
                }
                ParamEvent::Ack => {}
                ParamEvent::Nack { id } => {
                    let value = main_thread.param_snapshot.load().values[id];
                    let _ = main_thread.param_changes_tx.push(ParamChange { id, value });
                }
            }
        }

        let AudioThreadState {
            param_changes_rx,
            daw_events_tx,
            ..
        } = audio_thread;

        // Recover the tails of the audio-thread fr the next 'activate'
        main_thread.pending_param_changes_rx = Some(param_changes_rx);
        main_thread.pending_daw_events_tx = Some(daw_events_tx);
    }
}

// [audio-thread & active & !processing]
pub unsafe extern "C" fn start_processing(plugin: *const clap_plugin) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };

    if let Some(audio_thread) = plugin_ref.audio_thread.as_mut() {
        if audio_thread.thread_id.is_none() {
            audio_thread.thread_id = Some(std::thread::current().id());
        }
    }

    true
}

// [audio-thread & active & processing]
pub unsafe extern "C" fn stop_processing(plugin: *const clap_plugin) {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };

    let audio_thread = plugin_ref.audio_thread.as_mut().expect("Audio Thread not initialized");
    audio_thread.assert_audio_thread();

    audio_thread.thread_id = None;
}

// [audio-thread & active]
pub unsafe extern "C" fn reset(plugin: *const clap_plugin) {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };

    let audio_thread = plugin_ref.audio_thread.as_mut().expect("Audio Thread not initialized");
    audio_thread.assert_audio_thread();

    if let Some(nam_model) = audio_thread.nam_model.as_mut() {
        dsp::nam::ffi::dsp_reset(nam_model.pin_mut(), audio_thread.sample_rate, audio_thread.input_buf.len() as i32);
    }

    audio_thread.input_buf.fill(0.0);
    audio_thread.output_buf.fill(0.0);
    audio_thread.dc_filter.reset();
}

// [thread-safe]
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
    if unsafe { CStr::from_ptr(id) } == CLAP_EXT_GUI {
        return &GUI_EXT as *const _ as *const c_void;
    }

    std::ptr::null()
}

// [main-thread]
pub unsafe extern "C" fn on_main_thread(plugin: *const clap_plugin) {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };

    let main = plugin_ref.main_thread.as_mut().expect("main thread not initialized");
    main.assert_main_thread();

    let mut snapshot_dirty = false;
    let mut new_snapshot = *main.param_snapshot.load_full();

    // 1. Changes on the GUI
    while let Ok(change) = main.ui_changes_rx.pop() {
        new_snapshot.values[change.id] = change.value;
        // propagates into audio-thread
        let _ = main.param_changes_tx.push(change);
        snapshot_dirty = true;
    }

    // 2. Events from audio-thread (automation + acks + nacks)
    while let Ok(event) = main.daw_events_rx.pop() {
        match event {
            ParamEvent::Automation { id, value } => {
                new_snapshot.values[id] = value;
                snapshot_dirty = true;
            }
            ParamEvent::Ack => {
                // Host accepted — nothing to do
            }
            ParamEvent::Nack { id } => {
                // Host rejected — requeue for the next iter
                let value = new_snapshot.values[id];
                let _ = main.param_changes_tx.push(ParamChange { id, value });
            }
        }
    }

    if snapshot_dirty {
        main.param_snapshot.store(Arc::new(new_snapshot));
    }
}

// [audio-thread & active & processing]
pub unsafe extern "C" fn process(plugin: *const clap_plugin, process: *const clap_process_t) -> clap_process_status {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };
    let process_ref = unsafe { process.as_ref_unchecked() };

    let Some(audio_thread) = plugin_ref.audio_thread.as_mut() else {
        return CLAP_PROCESS_ERROR as clap_process_status;
    };

    audio_thread.assert_audio_thread();

    sync_main_to_audio(audio_thread, process_ref.out_events.cast_mut());

    let in_events = unsafe { process_ref.in_events.as_ref_unchecked() };
    let event_count = in_events.size.map(|f| unsafe { f(process_ref.in_events) }).unwrap_or_default();

    for i in 0..event_count {
        if let Some(get) = in_events.get {
            let event = unsafe { get(process_ref.in_events, i) };
            handle_clap_event(audio_thread, event);
        }
    }

    debug_assert!(process_ref.audio_inputs_count == 1);
    debug_assert!(process_ref.audio_outputs_count == 1);

    let audio_input = unsafe { *process_ref.audio_inputs.as_ref_unchecked().data32.offset(0) };
    let audio_output = unsafe { *process_ref.audio_outputs.as_mut_unchecked().data32.offset(0) };

    render_audio(audio_thread, audio_input, audio_output, process_ref.frames_count as usize);

    CLAP_PROCESS_CONTINUE as clap_process_status
}
