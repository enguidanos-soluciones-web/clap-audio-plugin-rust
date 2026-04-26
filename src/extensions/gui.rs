use baseview::{Size, Window, WindowOpenOptions, WindowScalePolicy};
use std::sync::Arc;

use crate::{
    clap::*,
    gui::{platform::make_parent_window, window_handler::WindowHandler},
    plugin::Plugin,
};

pub static GUI_EXT: clap_plugin_gui_t = clap_plugin_gui {
    is_api_supported: Some(is_api_supported),
    get_preferred_api: Some(get_preferred_api),
    create: Some(create),
    destroy: Some(destroy),
    set_scale: Some(set_scale),
    get_size: Some(get_size),
    can_resize: Some(can_resize),
    get_resize_hints: Some(get_resize_hints),
    adjust_size: Some(adjust_size),
    set_size: Some(set_size),
    set_parent: Some(set_parent),
    set_transient: Some(set_transient),
    suggest_title: Some(suggest_title),
    show: Some(show),
    hide: Some(hide),
};

pub unsafe extern "C" fn is_api_supported(_plugin: *const clap_plugin_t, api: *const std::ffi::c_char, is_floating: bool) -> bool {
    if is_floating {
        return false;
    }

    let api_str = unsafe { std::ffi::CStr::from_ptr(api) };

    #[cfg(target_os = "linux")]
    return api_str == CLAP_WINDOW_API_X11;

    #[cfg(target_os = "windows")]
    return api_str == CLAP_WINDOW_API_WIN32;

    #[cfg(target_os = "macos")]
    return api_str == CLAP_WINDOW_API_COCOA;

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let _ = api_str;
        false
    }
}

pub unsafe extern "C" fn get_preferred_api(
    _plugin: *const clap_plugin_t,
    api: *mut *const std::ffi::c_char,
    is_floating: *mut bool,
) -> bool {
    unsafe { *is_floating = false };
    #[cfg(target_os = "linux")]
    {
        unsafe { *api = CLAP_WINDOW_API_X11.as_ptr() };
        true
    }
    #[cfg(target_os = "windows")]
    {
        unsafe { *api = CLAP_WINDOW_API_WIN32.as_ptr() };
        return true;
    }
    #[cfg(target_os = "macos")]
    {
        unsafe { *api = CLAP_WINDOW_API_COCOA.as_ptr() };
        return true;
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let _ = api;
        false
    }
}

pub unsafe extern "C" fn create(_plugin: *const clap_plugin_t, _api: *const std::ffi::c_char, _is_floating: bool) -> bool {
    true
}

pub unsafe extern "C" fn destroy(plugin: *const clap_plugin_t) {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };
    plugin_ref.state.gui_window = None;
}

pub unsafe extern "C" fn set_scale(_plugin: *const clap_plugin_t, _scale: f64) -> bool {
    false
}

pub unsafe extern "C" fn get_size(plugin: *const clap_plugin_t, width: *mut u32, height: *mut u32) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };
    unsafe { *width = plugin_ref.state.gui_width };
    unsafe { *height = plugin_ref.state.gui_height };
    true
}

pub unsafe extern "C" fn can_resize(_plugin: *const clap_plugin_t) -> bool {
    true
}

pub unsafe extern "C" fn get_resize_hints(_plugin: *const clap_plugin_t, hints: *mut clap_gui_resize_hints_t) -> bool {
    let h = unsafe { hints.as_mut_unchecked() };
    h.can_resize_horizontally = true;
    h.can_resize_vertically = true;
    h.preserve_aspect_ratio = false;
    h.aspect_ratio_width = 0;
    h.aspect_ratio_height = 0;
    true
}

pub unsafe extern "C" fn adjust_size(_plugin: *const clap_plugin_t, _width: *mut u32, _height: *mut u32) -> bool {
    true
}

pub unsafe extern "C" fn set_size(plugin: *const clap_plugin_t, width: u32, height: u32) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };
    plugin_ref.state.gui_width = width;
    plugin_ref.state.gui_height = height;
    true
}

pub unsafe extern "C" fn set_parent(plugin: *const clap_plugin_t, window: *const clap_window_t) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };

    let raw_parent_window = unsafe { make_parent_window(window) };

    let width = plugin_ref.state.gui_width;
    let height = plugin_ref.state.gui_height;

    let parameters_rx = Arc::clone(&plugin_ref.state.parameters_rx);
    let parameters_wx = Arc::clone(&plugin_ref.state.parameters_wx);

    let handle = Window::open_parented(
        &raw_parent_window,
        WindowOpenOptions {
            title: "NAM Player".to_string(),
            size: Size::new(width as f64, height as f64),
            scale: WindowScalePolicy::SystemScaleFactor,
        },
        move |_window| WindowHandler::new(width, height, parameters_rx, parameters_wx),
    );

    plugin_ref.state.gui_window = Some(handle);

    true
}

pub unsafe extern "C" fn set_transient(_plugin: *const clap_plugin_t, _window: *const clap_window_t) -> bool {
    false
}

pub unsafe extern "C" fn suggest_title(_plugin: *const clap_plugin_t, _title: *const std::ffi::c_char) {}

pub unsafe extern "C" fn show(_plugin: *const clap_plugin_t) -> bool {
    true
}

pub unsafe extern "C" fn hide(_plugin: *const clap_plugin_t) -> bool {
    true
}
