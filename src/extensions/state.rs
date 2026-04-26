use std::{os::raw::c_void, sync::Arc};

use crate::{clap::*, parameters::any::PARAMS_COUNT, plugin::Plugin, processors::sync_audio_to_main::sync_audio_to_main};

pub static STATE_EXT: clap_plugin_state_t = clap_plugin_state {
    save: Some(save),
    load: Some(load),
};

pub extern "C" fn save(plugin: *const clap_plugin_t, stream: *const clap_ostream_t) -> bool {
    let plugin = unsafe { (*plugin).plugin_data as *mut Plugin };
    let plugin_ref = unsafe { plugin.as_mut_unchecked() };

    unsafe { sync_audio_to_main(plugin_ref) };

    let Ok(mut params) = plugin_ref.parameters_wx.try_lock() else {
        return false;
    };

    let stream_ref = unsafe { stream.as_ref_unchecked() };

    let Some(write) = stream_ref.write else {
        return false;
    };

    let bytes_write = unsafe {
        write(
            stream,
            plugin_ref.parameters_rx.load().main_thread_parameters.as_ptr() as *const c_void,
            (std::mem::size_of::<f32>() * PARAMS_COUNT) as u64,
        ) as usize
    };

    let success = bytes_write == std::mem::size_of::<f32>() * PARAMS_COUNT;

    if success {
        for n in 0..PARAMS_COUNT {
            params.main_thread_parameters_changed[n] = true;
        }

        plugin_ref.parameters_rx.store(Arc::new(*params));
    }

    success
}

pub extern "C" fn load(plugin: *const clap_plugin_t, stream: *const clap_istream_t) -> bool {
    let plugin = unsafe { (*plugin).plugin_data as *mut Plugin };
    let plugin_ref = unsafe { plugin.as_mut_unchecked() };
    let stream_ref = unsafe { stream.as_ref_unchecked() };

    let Some(read) = stream_ref.read else {
        return false;
    };

    let Ok(mut params) = plugin_ref.parameters_wx.try_lock() else {
        return false;
    };

    let bytes_read = unsafe {
        read(
            stream,
            params.main_thread_parameters.as_ptr() as *mut c_void,
            (std::mem::size_of::<f32>() * PARAMS_COUNT) as u64,
        ) as usize
    };

    let success = bytes_read == std::mem::size_of::<f32>() * PARAMS_COUNT;
    if success {
        for n in 0..PARAMS_COUNT {
            params.main_thread_parameters_changed[n] = true;
        }

        plugin_ref.parameters_rx.store(Arc::new(*params));
    }

    success
}
