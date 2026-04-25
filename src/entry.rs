use crate::{clap::*, factory::PLUGIN_FACTORY, version::CLAP_VERSION_INIT};
use std::ffi::{CStr, c_char, c_void};

#[unsafe(no_mangle)]
pub static clap_entry: clap_plugin_entry_t = clap_plugin_entry {
    clap_version: CLAP_VERSION_INIT,
    init: Some(entry_init),
    deinit: Some(entry_deinit),
    get_factory: Some(entry_get_factory),
};

unsafe extern "C" fn entry_init(_plugin_path: *const c_char) -> bool {
    true
}

unsafe extern "C" fn entry_deinit() {}

unsafe extern "C" fn entry_get_factory(factory_id: *const c_char) -> *const c_void {
    unsafe {
        if CStr::from_ptr(factory_id) == CLAP_PLUGIN_FACTORY_ID {
            return &PLUGIN_FACTORY as *const _ as *const c_void;
        }
    }

    std::ptr::null()
}
