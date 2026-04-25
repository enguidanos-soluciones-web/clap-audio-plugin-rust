use crate::{
    clap::*,
    gui::parameters::any::{AnyParameter, PARAMS_COUNT},
    helper::copy_cstr,
    plugin::Plugin,
    processors::{handle_clap_event::handle_clap_event, sync_main_to_audio::sync_main_to_audio},
};
use std::ffi::c_char;
use std::io::Write;

pub static PARAMETERS_EXT: clap_plugin_params_t = clap_plugin_params {
    count: Some(count),
    get_info: Some(get_info),
    get_value: Some(get_value),
    value_to_text: Some(value_to_text),
    text_to_value: Some(text_to_value),
    flush: Some(flush),
};

pub extern "C" fn count(_plugin: *const clap_plugin_t) -> u32 {
    PARAMS_COUNT as u32
}

pub extern "C" fn get_info(_plugin: *const clap_plugin_t, index: u32, information: *mut clap_param_info_t) -> bool {
    let Ok(param) = AnyParameter::try_from(index as usize) else {
        return false;
    };

    let mut new_information = unsafe { std::mem::zeroed::<clap_param_info_t>() };
    new_information.id = index;
    new_information.flags = CLAP_PARAM_IS_AUTOMATABLE;

    match &param {
        AnyParameter::InputGain { inner } => {
            new_information.min_value = inner.behave.min;
            new_information.max_value = inner.behave.max;
            new_information.default_value = inner.behave.def;
            copy_cstr(&mut new_information.name, inner.name.as_bytes());
        }
        AnyParameter::OutputGain { inner } => {
            new_information.min_value = inner.behave.min;
            new_information.max_value = inner.behave.max;
            new_information.default_value = inner.behave.def;
            copy_cstr(&mut new_information.name, inner.name.as_bytes());
        }
    }

    unsafe { std::ptr::write(information, new_information) };

    true
}

pub extern "C" fn get_value(plugin: *const clap_plugin_t, id: clap_id, value: *mut f64) -> bool {
    let plugin = unsafe { (*plugin).plugin_data as *const Plugin };
    let plugin_ref = unsafe { plugin.as_ref_unchecked() };

    if id as usize >= PARAMS_COUNT {
        return false;
    }

    let value_ref = unsafe { value.as_mut_unchecked() };
    let params = plugin_ref.parameters_rx.load();

    *value_ref = if params.main_thread_parameters_changed[id as usize] {
        params.main_thread_parameters[id as usize] as f64
    } else {
        params.audio_thread_parameters[id as usize] as f64
    };

    true
}

pub extern "C" fn value_to_text(_plugin: *const clap_plugin_t, id: clap_id, value: f64, display: *mut c_char, size: u32) -> bool {
    if id as usize >= PARAMS_COUNT {
        return false;
    }

    let buffer = unsafe { std::slice::from_raw_parts_mut(display as *mut u8, size as usize) };
    let mut cursor = std::io::Cursor::new(buffer);

    write!(cursor, "{}\0", value).is_ok()
}

pub extern "C" fn text_to_value(_plugin: *const clap_plugin_t, _param_id: clap_id, display: *const c_char, value: *mut f64) -> bool {
    let Ok(s) = (unsafe { std::ffi::CStr::from_ptr(display) }).to_str() else {
        return false;
    };
    let Ok(parsed) = s.trim().parse::<f64>() else {
        return false;
    };
    unsafe { *value = parsed };
    true
}

pub extern "C" fn flush(plugin: *const clap_plugin_t, inn: *const clap_input_events_t, out: *const clap_output_events_t) {
    let plugin = unsafe { (*plugin).plugin_data as *mut Plugin };
    let plugin_ref = unsafe { plugin.as_mut_unchecked() };
    let in_ref = unsafe { inn.as_ref_unchecked() };

    let mut event_count = 0;

    if let Some(size) = in_ref.size {
        unsafe { event_count = size(inn) };
    }

    for n in 0..event_count {
        if let Some(get) = in_ref.get {
            let event = unsafe { get(inn, n) };
            unsafe { handle_clap_event(plugin_ref, event) };
        }
    }

    unsafe { sync_main_to_audio(plugin_ref, out) };
}
