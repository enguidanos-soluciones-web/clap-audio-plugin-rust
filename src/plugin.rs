// Thread and channel architecture
//
//  ┌─────────────────────────────────────────────────────────────────────────────────┐
//  │ GUI THREAD  (baseview)                                                          │
//  │  WindowHandler                                                                  │
//  │  · on_frame  — reads  param_snapshot (ArcSwap, lock-free)                       │
//  │  · on_event  — writes gui_changes Sender<ParamChange>                           │
//  │              — calls  host.request_callback() [thread-safe]                     │
//  └──────────────────────────────┬──────────────────────────────────────────────────┘
//                                 │ gui_changes
//                                 │ Sender<ParamChange>  ───────────────────────────-┐
//                                 ▼                                                  │
//  ┌─────────────────────────────────────────────────────────────────────────────────┤
//  │ MAIN THREAD  (host-managed)                                                     │
//  │  on_main_thread()                                                               │
//  │  · drains gui_changes  Receiver<ParamChange>  → updates param_snapshot          │
//  │                                               → pushes  param_changes           │
//  │  · drains daw_events   Receiver<ParamEvent>                                     │
//  │      Automation  → updates param_snapshot                                       │
//  │      Ack         → (no-op)                                                      │
//  │      Nack        → requeues into param_changes                                  │
//  └──────────────────────────────┬──────────────────────────────────────────────────┘
//                                 │ param_changes
//                                 │ Sender<ParamChange>  ─────────────────────────── ┐
//                                 ▼                                                  │
//  ┌─────────────────────────────────────────────────────────────────────────────────┤
//  │ AUDIO THREAD  (host-managed, real-time)                                         │
//  │  process()                                                                      │
//  │  · sync_main_to_audio — drains param_changes Receiver<ParamChange>              │
//  │                       — emits  CLAP_EVENT_PARAM_VALUE → host out_events         │
//  │                       — pushes daw_events Sender<ParamEvent> (Ack / Nack)       │
//  │  · handle_clap_event  — reads  CLAP_EVENT_PARAM_VALUE from host in_events       │
//  │                       — pushes daw_events Sender<ParamEvent> (Automation)       │
//  │                       — calls  host.request_callback() [thread-safe]            │
//  │  · render_audio       — reads  param_snapshot (ArcSwap, lock-free)              │
//  └─────────────────────────────────────────────────────────────────────────────────┘
//
//  Shared state (Arc<ArcSwap<ParamSnapshot>>)
//  · written by main thread  in on_main_thread()
//  · read    by audio thread in render_audio()        (lock-free load)
//  · read    by GUI thread   in on_frame()            (lock-free load)

use crate::{
    channel::channel,
    clap::*,
    descriptor::PLUGIN_DESCRIPTOR,
    dsp::{self, dc_filter::DcFilter, klon_buffer::KlonBuffer, lowpass_filter::LowPassFilter},
    extensions::{audio_ports::AUDIO_PORTS_EXT, gui::GUI_EXT, parameters::PARAMETERS_EXT, state::STATE_EXT},
    helper::{DecibelConversion, db_to_linear},
    parameters::any::PARAMS_COUNT,
    plugin,
    processors::{
        handle_clap_event::handle_clap_event,
        render_audio::{render_audio_f32, render_audio_f64},
        sync_main_to_audio::sync_main_to_audio,
    },
    state::{AudioThreadState, GuiRequest, MainThreadState, ModelUpdate, ParamChange, ParamEvent, ParamSnapshot},
};
use arc_swap::ArcSwap;
use std::{
    ffi::{CStr, c_char, c_void},
    sync::Arc,
};

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

    let (param_changes_tx, _) = channel::<ParamChange>(64);
    let (_, daw_events_rx) = channel::<ParamEvent>(64);
    let (_, gui_changes_rx) = channel::<ParamChange>(64);

    let param_snapshot = Arc::new(ArcSwap::new(Arc::new(ParamSnapshot {
        values: [0.0; PARAMS_COUNT],
    })));

    let (model_updates_tx, _model_updates_rx) = channel::<ModelUpdate>(1);
    let (_, gui_requests_rx) = channel::<GuiRequest>(4);

    plugin_ref.main_thread = Some(MainThreadState {
        param_snapshot,
        daw_events: daw_events_rx,
        gui_changes: gui_changes_rx,
        param_changes: param_changes_tx,
        gui_shared: Default::default(),
        gui_window: None,
        gui_width: 800,
        gui_height: 400,
        model_updates: model_updates_tx,
        gui_requests: gui_requests_rx,
        selected_model_path: None,
        thread_id: Some(std::thread::current().id()),
    });

    let main_thread = plugin_ref.main_thread.as_mut().unwrap();
    let mut default_values = [0.0; PARAMS_COUNT];
    for n in 0..PARAMS_COUNT {
        let mut information = unsafe { std::mem::zeroed::<clap_param_info_t>() };
        if let Some(get_info) = PARAMETERS_EXT.get_info {
            // SAFETY: MAIN-THREAD must have FIRST THREAD_ID as SOME. OTHERWISE get_info WILL PANIC.
            if unsafe { get_info(plugin, n as u32, &mut information) } {
                default_values[n] = information.default_value;
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
        return true;
    }

    let main_thread = plugin_ref.main_thread.as_mut().expect("main thread not initialized");
    main_thread.assert_main_thread();

    if let Some(path) = main_thread.selected_model_path.clone() {
        if let Ok(json) = std::fs::read_to_string(&path) {
            let model_name = std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&path)
                .to_string();

            let mut nam_model = dsp::nam::ffi::load(&json);
            dsp::nam::ffi::reset_and_prewarm(nam_model.pin_mut(), sample_rate, max_frames_count as i32);

            const NAM_TARGET_LOUDNESS_DBFS: f64 = -12.0;
            let loudness_correction = if dsp::nam::ffi::has_loudness(&nam_model) {
                db_to_linear(
                    NAM_TARGET_LOUDNESS_DBFS - dsp::nam::ffi::get_loudness(&nam_model),
                    DecibelConversion::Amplitude,
                )
            } else {
                1.0
            };

            let model_rate = dsp::nam::ffi::get_sample_rate_from_nam_file(&json);

            let mut new_gui_shared = main_thread.gui_shared.load_full().as_ref().clone();
            new_gui_shared.nam_model_rate = Some(model_rate);
            new_gui_shared.model_name = Some(model_name);
            main_thread.gui_shared.store(Arc::new(new_gui_shared));

            let _ = main_thread.model_updates.push(ModelUpdate {
                model: nam_model,
                loudness_correction,
            });
        }
    }

    plugin_ref.audio_thread = Some(AudioThreadState {
        host: plugin_ref.host,
        sample_rate,
        nam_model: None,
        nam_loudness_correction: 1.0,
        dc_filter: DcFilter::new(20.0, sample_rate),
        klon_buffer: KlonBuffer::new(sample_rate),
        lowpass_filter: LowPassFilter::new(16.0, sample_rate),
        input_buf: vec![0.0; max_frames_count as usize],
        output_buf: vec![0.0; max_frames_count as usize],
        model_updates: main_thread.model_updates.new_receiver(),
        param_snapshot: Arc::clone(&main_thread.param_snapshot),
        param_changes: main_thread.param_changes.new_receiver(),
        daw_events: main_thread.daw_events.new_sender(),
        thread_id: None,
    });

    true
}

// [main-thread & active]
pub unsafe extern "C" fn deactivate(plugin: *const clap_plugin) {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };

    if plugin_ref.audio_thread.is_none() {
        return;
    }

    let main_thread = plugin_ref.main_thread.as_mut().expect("main thread not initialized");
    main_thread.assert_main_thread();

    // Drain pending events from audio before drop because we don't want to
    // loose the automatization that arrived just before 'deactivate'.
    while let Some(event) = main_thread.daw_events.pop() {
        match event {
            ParamEvent::Automation { id, value } => {
                let mut new_snapshot = *main_thread.param_snapshot.load_full();
                new_snapshot.values[id] = value;
                main_thread.param_snapshot.store(Arc::new(new_snapshot));
            }
            ParamEvent::Ack => {}
            ParamEvent::Nack { id } => {
                let value = main_thread.param_snapshot.load().values[id];
                let _ = main_thread.param_changes.push(ParamChange { id, value });
            }
        }
    }

    plugin_ref.audio_thread.take();
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
    audio_thread.reset();
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
    while let Some(change) = main.gui_changes.pop() {
        new_snapshot.values[change.id] = change.value;
        // propagates into audio-thread
        let _ = main.param_changes.push(change);
        snapshot_dirty = true;
    }

    // 2. Events from audio-thread (automation + acks + nacks)
    while let Some(event) = main.daw_events.pop() {
        match event {
            ParamEvent::Automation { id, value } => {
                new_snapshot.values[id] = value;
                snapshot_dirty = true;
            }
            ParamEvent::Ack => {}
            ParamEvent::Nack { id } => {
                let value = new_snapshot.values[id];
                let _ = main.param_changes.push(ParamChange { id, value });
            }
        }
    }

    // 3. GUI requests (e.g. open file browser to load a NAM model)
    while let Some(request) = main.gui_requests.pop() {
        match request {
            GuiRequest::OpenFileBrowser => {
                let Some(path) = rfd::FileDialog::new().add_filter("NAM model", &["nam"]).pick_file() else {
                    continue;
                };

                let Ok(json) = std::fs::read_to_string(&path) else {
                    continue;
                };

                let mut model = dsp::nam::ffi::load(&json);

                const NAM_TARGET_LOUDNESS_DBFS: f64 = -12.0;
                let loudness_correction = if dsp::nam::ffi::has_loudness(&model) {
                    let model_loudness = dsp::nam::ffi::get_loudness(&model);
                    db_to_linear(NAM_TARGET_LOUDNESS_DBFS - model_loudness, DecibelConversion::Amplitude)
                } else {
                    1.0
                };

                let model_rate = dsp::nam::ffi::get_sample_rate_from_nam_file(&json);
                let model_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();
                main.selected_model_path = Some(path.to_string_lossy().into_owned());

                // Reset and prewarm using the current audio thread sample rate and buffer size.
                // If the audio thread is not yet active we skip — the model will be loaded fresh on activate().
                if let Some(audio) = plugin_ref.audio_thread.as_mut() {
                    dsp::nam::ffi::reset_and_prewarm(model.pin_mut(), audio.sample_rate, audio.input_buf.len() as i32);
                } else {
                    continue;
                }

                // Update GUI with new model info before pushing to audio thread.
                let mut new_gui_shared = main.gui_shared.load_full().as_ref().clone();
                new_gui_shared.nam_model_rate = Some(model_rate);
                new_gui_shared.model_name = Some(model_name.clone());
                main.gui_shared.store(Arc::new(new_gui_shared));

                // Drain any stale pending model so the audio thread always gets the latest.
                let _ = main.model_updates.push(ModelUpdate {
                    model,
                    loudness_correction,
                });
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
    if audio_thread.thread_id.is_none() {
        return CLAP_PROCESS_ERROR as clap_process_status;
    }

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

    // Some hosts (e.g. REAPER on macOS) may call process() before audio buffers
    // are fully set up, which is technically outside the CLAP spec but happens
    // in practice. Guard against null pointers to avoid a crash.
    if process_ref.audio_inputs_count == 0 || process_ref.audio_outputs_count == 0 {
        return CLAP_PROCESS_CONTINUE as clap_process_status;
    }

    let audio_inputs = unsafe { process_ref.audio_inputs.as_ref() };
    let audio_outputs = unsafe { process_ref.audio_outputs.as_mut() };

    let (Some(audio_inputs), Some(audio_outputs)) = (audio_inputs, audio_outputs) else {
        return CLAP_PROCESS_CONTINUE as clap_process_status;
    };

    let nframes = process_ref.frames_count as usize;

    if !audio_inputs.data64.is_null() && !audio_outputs.data64.is_null() {
        let input = unsafe { *audio_inputs.data64.offset(0) };
        let output = unsafe { *audio_outputs.data64.offset(0) };
        render_audio_f64(audio_thread, input, output, nframes);
        return CLAP_PROCESS_CONTINUE as clap_process_status;
    }
    if !audio_inputs.data32.is_null() && !audio_outputs.data32.is_null() {
        let input = unsafe { *audio_inputs.data32.offset(0) };
        let output = unsafe { *audio_outputs.data32.offset(0) };
        render_audio_f32(audio_thread, input, output, nframes);
        return CLAP_PROCESS_CONTINUE as clap_process_status;
    }

    CLAP_PROCESS_CONTINUE as clap_process_status
}
