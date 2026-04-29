use crate::{clap::*, parameters::any::PARAMS_COUNT, plugin::Plugin, state::ParamChange};
use std::{os::raw::c_void, sync::Arc};

pub static STATE_EXT: clap_plugin_state_t = clap_plugin_state {
    save: Some(save),
    load: Some(load),
};

// [main-thread]
pub extern "C" fn save(plugin: *const clap_plugin_t, stream: *const clap_ostream_t) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main = plugin_ref.main_thread.as_ref().unwrap();
    main.assert_main_thread();

    let stream_ref = unsafe { stream.as_ref_unchecked() };
    let Some(write) = stream_ref.write else {
        return false;
    };

    let snapshot = main.param_snapshot.load();

    let write_all = |buf: &[u8]| -> bool {
        let mut offset = 0;
        while offset < buf.len() {
            let n = unsafe { write(stream, buf.as_ptr().add(offset) as *const c_void, (buf.len() - offset) as u64) };
            if n <= 0 { return false; }
            offset += n as usize;
        }
        true
    };

    let values_bytes = unsafe {
        std::slice::from_raw_parts(snapshot.values.as_ptr() as *const u8, std::mem::size_of::<f64>() * PARAMS_COUNT)
    };
    if !write_all(values_bytes) { return false; }

    let path = main.selected_model_path.as_deref().unwrap_or("");
    let path_len = (path.len() as u32).to_le_bytes();
    if !write_all(&path_len) { return false; }
    if !write_all(path.as_bytes()) { return false; }

    true
}

// [main-thread]
pub extern "C" fn load(plugin: *const clap_plugin_t, stream: *const clap_istream_t) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };

    let main_thread = plugin_ref.main_thread.as_mut().unwrap();
    main_thread.assert_main_thread();

    let stream_ref = unsafe { stream.as_ref_unchecked() };
    let Some(read) = stream_ref.read else {
        return false;
    };

    let read_all = |buf: &mut [u8]| -> bool {
        let mut offset = 0;
        while offset < buf.len() {
            let n = unsafe { read(stream, buf.as_mut_ptr().add(offset) as *mut c_void, (buf.len() - offset) as u64) };
            if n <= 0 { return false; }
            offset += n as usize;
        }
        true
    };

    let mut new_snapshot = *main_thread.param_snapshot.load_full();
    let values_bytes = unsafe {
        std::slice::from_raw_parts_mut(new_snapshot.values.as_mut_ptr() as *mut u8, std::mem::size_of::<f64>() * PARAMS_COUNT)
    };
    if !read_all(values_bytes) { return false; }

    main_thread.param_snapshot.store(Arc::new(new_snapshot));
    for id in 0..PARAMS_COUNT {
        let _ = main_thread.param_changes.push(ParamChange { id, value: new_snapshot.values[id] });
    }

    // Read model path (optional — older state blobs without path are still valid).
    let mut len_buf = [0u8; 4];
    if read_all(&mut len_buf) {
        let path_len = u32::from_le_bytes(len_buf) as usize;
        let mut path_bytes = vec![0u8; path_len];
        if read_all(&mut path_bytes) {
            main_thread.selected_model_path = String::from_utf8(path_bytes).ok().filter(|s| !s.is_empty());
        }
    }

    true
}
