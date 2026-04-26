use crate::{
    clap::*,
    descriptor::PLUGIN_DESCRIPTOR,
    plugin::{PLUGIN_CLASS, Plugin},
    version::clap_version_is_compatible,
};
use std::ffi::{CStr, c_char, c_void};

pub static PLUGIN_FACTORY: clap_plugin_factory_t = clap_plugin_factory {
    get_plugin_count: Some(get_plugin_count),
    get_plugin_descriptor: Some(get_plugin_descriptor),
    create_plugin: Some(create_plugin),
};

unsafe extern "C" fn get_plugin_count(_factory: *const clap_plugin_factory) -> u32 {
    1
}

unsafe extern "C" fn get_plugin_descriptor(_factory: *const clap_plugin_factory, index: u32) -> *const clap_plugin_descriptor_t {
    if index == 0 {
        return &PLUGIN_DESCRIPTOR;
    }

    std::ptr::null()
}

unsafe extern "C" fn create_plugin(
    _factory: *const clap_plugin_factory,
    host: *const clap_host_t,
    plugin_id: *const c_char,
) -> *const clap_plugin_t {
    let host_ref = unsafe { host.as_ref_unchecked() };

    unsafe {
        if !clap_version_is_compatible(host_ref.clap_version) || CStr::from_ptr(plugin_id) != CStr::from_ptr(PLUGIN_DESCRIPTOR.id) {
            return std::ptr::null();
        }
    }

    let plugin = Box::new(Plugin {
        inner: PLUGIN_CLASS,
        host,
        sample_rate: 0.,
        model: None,
        input_buf: Vec::new(),
        output_buf: Vec::new(),
        parameters_rx: Default::default(),
        parameters_wx: Default::default(),
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        gui_window: None,
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        gui_width: 600,
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        gui_height: 400,
    });

    let raw = Box::into_raw(plugin);

    unsafe {
        (*raw).inner.plugin_data = raw as *mut c_void;
        &(*raw).inner as *const clap_plugin
    }
}
