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

    let total = std::mem::size_of::<f64>() * PARAMS_COUNT;
    let buf = unsafe { std::slice::from_raw_parts(snapshot.values.as_ptr() as *const u8, total) };
    let mut offset = 0;
    while offset < total {
        let n = unsafe { write(stream, buf.as_ptr().add(offset) as *const c_void, (total - offset) as u64) };
        if n <= 0 {
            break;
        }
        offset += n as usize;
    }

    offset == total
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

    let total = std::mem::size_of::<f64>() * PARAMS_COUNT;
    let buf = unsafe { std::slice::from_raw_parts_mut(new_snapshot.values.as_mut_ptr() as *mut u8, total) };
    let mut offset = 0;
    while offset < total {
        let n = unsafe { read(stream, buf.as_mut_ptr().add(offset) as *mut c_void, (total - offset) as u64) };
        if n <= 0 {
            break;
        }
        offset += n as usize;
    }

    let success = offset == total;

    if success {
        // Pub new snapshot with loaded values
        main_thread.param_snapshot.store(Arc::new(new_snapshot));

        // Propagate all the params into audio-thread
        for id in 0..PARAMS_COUNT {
            let _ = main_thread.param_changes.push(ParamChange {
                id,
                value: new_snapshot.values[id],
            });
        }
    }

    success
}
