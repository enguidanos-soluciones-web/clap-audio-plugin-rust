use baseview::{Size, Window, WindowOpenOptions, WindowScalePolicy};
use std::sync::Arc;

use crate::{
    clap::*,
    gui::window_handler::WindowHandler,
    plugin::Plugin,
    state::MainThreadState,
};

#[cfg(feature = "resize")]
use crate::gui::platform::{extract_parent_handle, make_parent_window_from_usize};

#[cfg(not(feature = "resize"))]
use crate::gui::platform::ClapParentWindow;

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

// [main-thread]
pub unsafe extern "C" fn is_api_supported(plugin: *const clap_plugin_t, api: *const std::ffi::c_char, is_floating: bool) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main_thread = plugin_ref.main_thread.as_ref().expect("Main Thread not initialized");
    main_thread.assert_main_thread();

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

// [main-thread]
pub unsafe extern "C" fn get_preferred_api(
    plugin: *const clap_plugin_t,
    api: *mut *const std::ffi::c_char,
    is_floating: *mut bool,
) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main_thread = plugin_ref.main_thread.as_ref().expect("Main Thread not initialized");
    main_thread.assert_main_thread();

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

// [main-thread]
pub unsafe extern "C" fn create(plugin: *const clap_plugin_t, _api: *const std::ffi::c_char, _is_floating: bool) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main_thread = plugin_ref.main_thread.as_ref().expect("Main Thread not initialized");
    main_thread.assert_main_thread();

    true
}

// [main-thread]
pub unsafe extern "C" fn destroy(plugin: *const clap_plugin_t) {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };

    let main_thread = plugin_ref.main_thread.as_mut().expect("Main Thread not initialized");
    main_thread.assert_main_thread();

    if let Some(window) = main_thread.gui_window.as_mut() {
        window.close();
    }

    main_thread.gui_window = None;
}

// [main-thread]
pub unsafe extern "C" fn set_scale(plugin: *const clap_plugin_t, _scale: f64) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main_thread = plugin_ref.main_thread.as_ref().expect("Main Thread not initialized");
    main_thread.assert_main_thread();

    false
}

// [main-thread]
pub unsafe extern "C" fn get_size(plugin: *const clap_plugin_t, width: *mut u32, height: *mut u32) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main_thread = plugin_ref.main_thread.as_ref().expect("Main Thread not initialized");
    main_thread.assert_main_thread();

    unsafe { *width = main_thread.gui_width };
    unsafe { *height = main_thread.gui_height };

    true
}

// [main-thread & !floating]
pub unsafe extern "C" fn can_resize(plugin: *const clap_plugin_t) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main_thread = plugin_ref.main_thread.as_ref().expect("Main Thread not initialized");
    main_thread.assert_main_thread();

    #[cfg(feature = "resize")]
    return true;

    #[cfg(not(feature = "resize"))]
    false
}

// [main-thread & !floating]
pub unsafe extern "C" fn get_resize_hints(plugin: *const clap_plugin_t, hints: *mut clap_gui_resize_hints_t) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main_thread = plugin_ref.main_thread.as_ref().expect("Main Thread not initialized");
    main_thread.assert_main_thread();

    #[cfg(not(feature = "resize"))]
    {
        let _ = hints;
        return false;
    }

    #[cfg(feature = "resize")]
    {
        let h = unsafe { hints.as_mut_unchecked() };
        h.can_resize_horizontally = true;
        h.can_resize_vertically = true;
        h.preserve_aspect_ratio = false;
        h.aspect_ratio_width = 0;
        h.aspect_ratio_height = 0;
        true
    }
}

// [main-thread & !floating]
pub unsafe extern "C" fn adjust_size(plugin: *const clap_plugin_t, _width: *mut u32, _height: *mut u32) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main_thread = plugin_ref.main_thread.as_ref().expect("Main Thread not initialized");
    main_thread.assert_main_thread();

    true
}

// [main-thread & !floating]
pub unsafe extern "C" fn set_parent(plugin: *const clap_plugin_t, window: *const clap_window_t) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };

    let main_thread = plugin_ref.main_thread.as_mut().expect("Main Thread not initialized");
    main_thread.assert_main_thread();

    #[cfg(feature = "resize")]
    {
        main_thread.gui_parent = unsafe { extract_parent_handle(window) };
    }

    let host_addr = plugin_ref.host as usize;

    #[cfg(feature = "resize")]
    open_window(main_thread, host_addr);

    #[cfg(not(feature = "resize"))]
    {
        let raw_parent = unsafe { crate::gui::platform::make_parent_window(window) };
        open_window_with_parent(main_thread, raw_parent, host_addr);
    }

    true
}

pub unsafe extern "C" fn set_size(plugin: *const clap_plugin_t, width: u32, height: u32) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };

    let main_thread = plugin_ref.main_thread.as_mut().expect("Main Thread not initialized");
    main_thread.assert_main_thread();

    main_thread.gui_width = width;
    main_thread.gui_height = height;

    #[cfg(feature = "resize")]
    if main_thread.gui_window.is_some() && main_thread.gui_parent != 0 {
        main_thread.gui_needs_reopen = true;
        // request_callback schedules on_main_thread — reopen happens there
        if let Some(request_callback) = unsafe { *plugin_ref.host }.request_callback {
            unsafe { request_callback(plugin_ref.host) };
        }
    }

    true
}

#[cfg(feature = "resize")]
pub fn open_window(main_thread: &mut MainThreadState, host_addr: usize) {
    let raw_parent = unsafe { make_parent_window_from_usize(main_thread.gui_parent) };

    let width = main_thread.gui_width;
    let height = main_thread.gui_height;
    let params_snapshot = Arc::clone(&main_thread.param_snapshot);
    let gui_shared = Arc::clone(&main_thread.gui_shared);
    let gui_requests = main_thread.gui_requests.new_sender();

    let handle = Window::open_parented(
        &raw_parent,
        WindowOpenOptions {
            title: "Neural Amp Modeler".to_string(),
            size: Size::new(width as f64, height as f64),
            scale: WindowScalePolicy::SystemScaleFactor,
        },
        move |window| {
            WindowHandler::new(
                window,
                width,
                height,
                host_addr as *const clap_host_t,
                gui_shared,
                gui_requests,
                params_snapshot,
            )
        },
    );

    main_thread.gui_window = Some(handle);
}

#[cfg(not(feature = "resize"))]
pub fn open_window_with_parent(main_thread: &mut MainThreadState, raw_parent: ClapParentWindow, host_addr: usize) {
    let width = main_thread.gui_width;
    let height = main_thread.gui_height;
    let params_snapshot = Arc::clone(&main_thread.param_snapshot);
    let gui_shared = Arc::clone(&main_thread.gui_shared);
    let gui_requests = main_thread.gui_requests.new_sender();

    let handle = Window::open_parented(
        &raw_parent,
        WindowOpenOptions {
            title: "Neural Amp Modeler".to_string(),
            size: Size::new(width as f64, height as f64),
            scale: WindowScalePolicy::SystemScaleFactor,
        },
        move |window| {
            WindowHandler::new(
                window,
                width,
                height,
                host_addr as *const clap_host_t,
                gui_shared,
                gui_requests,
                params_snapshot,
            )
        },
    );

    main_thread.gui_window = Some(handle);
}

// [main-thread & floating]
pub unsafe extern "C" fn set_transient(plugin: *const clap_plugin_t, _window: *const clap_window_t) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main_thread = plugin_ref.main_thread.as_ref().expect("Main Thread not initialized");
    main_thread.assert_main_thread();

    false
}

// [main-thread & floating]
pub unsafe extern "C" fn suggest_title(plugin: *const clap_plugin_t, _title: *const std::ffi::c_char) {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main_thread = plugin_ref.main_thread.as_ref().expect("Main Thread not initialized");
    main_thread.assert_main_thread();
}

// [main-thread]
pub unsafe extern "C" fn show(plugin: *const clap_plugin_t) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main_thread = plugin_ref.main_thread.as_ref().expect("Main Thread not initialized");
    main_thread.assert_main_thread();

    true
}

// [main-thread]
pub unsafe extern "C" fn hide(plugin: *const clap_plugin_t) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };

    let main_thread = plugin_ref.main_thread.as_ref().expect("Main Thread not initialized");
    main_thread.assert_main_thread();

    true
}
