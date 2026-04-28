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
    let bytes_written = unsafe {
        write(
            stream,
            snapshot.values.as_ptr() as *const c_void,
            (std::mem::size_of::<f32>() * PARAMS_COUNT) as u64,
        ) as usize
    };

    bytes_written == std::mem::size_of::<f32>() * PARAMS_COUNT
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

    let mut new_snapshot = *main_thread.param_snapshot.load_full();

    let bytes_read = unsafe {
        read(
            stream,
            new_snapshot.values.as_mut_ptr() as *mut c_void,
            (std::mem::size_of::<f32>() * PARAMS_COUNT) as u64,
        ) as usize
    };

    let success = bytes_read == std::mem::size_of::<f32>() * PARAMS_COUNT;

    if success {
        // Pub new snapshot with loaded values
        main_thread.param_snapshot.store(Arc::new(new_snapshot));

        // Propagate all the params into audio-thread
        for id in 0..PARAMS_COUNT {
            let _ = main_thread.param_changes_tx.push(ParamChange {
                id,
                value: new_snapshot.values[id],
            });
        }
    }

    success
}
