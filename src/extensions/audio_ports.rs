use crate::{clap::*, helper::copy_cstr, plugin::Plugin};

pub static AUDIO_PORTS_EXT: clap_plugin_audio_ports_t = clap_plugin_audio_ports {
    count: Some(count_audio_ports),
    get: Some(get_audio_ports),
};

// [main-thread]
pub unsafe extern "C" fn count_audio_ports(plugin: *const clap_plugin_t, _is_input: bool) -> u32 {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main_thread = plugin_ref.main_thread.as_ref().expect("main thread not initialized");
    main_thread.assert_main_thread();

    1
}

// [main-thread]
pub unsafe extern "C" fn get_audio_ports(
    plugin: *const clap_plugin_t,
    index: u32,
    is_input: bool,
    info: *mut clap_audio_port_info_t,
) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main_thread = plugin_ref.main_thread.as_ref().expect("main thread not initialized");
    main_thread.assert_main_thread();

    if index != 0 {
        return false;
    }

    let info_ref = unsafe { info.as_mut_unchecked() };

    info_ref.id = if is_input { 0 } else { 1 };
    info_ref.channel_count = 1;
    info_ref.flags = CLAP_AUDIO_PORT_IS_MAIN;
    info_ref.port_type = CLAP_PORT_MONO.as_ptr();
    info_ref.in_place_pair = CLAP_INVALID_ID;

    copy_cstr(&mut info_ref.name, if is_input { b"Audio Input" } else { b"Audio Output" });

    true
}
