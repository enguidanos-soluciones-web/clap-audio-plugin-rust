use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use raw_window_handle::{HasWindowHandle, RawWindowHandle, WindowHandle};
use wry::{
    Rect, WebViewBuilder,
    dpi::{LogicalPosition, LogicalSize},
};

use crate::{
    clap::*,
    plugin::{Plugin, PluginParameters},
    processors::{
        handle_ui_event::handle_ui_event,
        send_ui_event::{GUIRequest, send_ui_event},
    },
};

#[cfg(target_os = "windows")]
struct HostWindow(std::num::NonZeroIsize);

#[cfg(target_os = "windows")]
impl HasWindowHandle for HostWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, raw_window_handle::HandleError> {
        use raw_window_handle::Win32WindowHandle;
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Win32(Win32WindowHandle::new(self.0))) })
    }
}

#[cfg(target_os = "macos")]
struct HostWindow(std::ptr::NonNull<std::ffi::c_void>);

#[cfg(target_os = "macos")]
impl HasWindowHandle for HostWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, raw_window_handle::HandleError> {
        use raw_window_handle::AppKitWindowHandle;
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::AppKit(AppKitWindowHandle::new(self.0))) })
    }
}

unsafe fn host_window_from_clap(window: *const clap_window_t) -> Option<HostWindow> {
    #[cfg(target_os = "windows")]
    {
        let hwnd = unsafe { (*window).__bindgen_anon_1.win32 } as isize;
        std::num::NonZeroIsize::new(hwnd).map(HostWindow)
    }
    #[cfg(target_os = "macos")]
    {
        let cocoa = unsafe { (*window).__bindgen_anon_1.cocoa };
        std::ptr::NonNull::new(cocoa as *mut std::ffi::c_void).map(HostWindow)
    }
}

const GUI_HTML: &str = include_str!("../gui/entry.html");

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
    #[cfg(target_os = "windows")]
    return api_str == CLAP_WINDOW_API_WIN32;
    #[cfg(target_os = "macos")]
    return api_str == CLAP_WINDOW_API_COCOA;
}

pub unsafe extern "C" fn get_preferred_api(
    _plugin: *const clap_plugin_t,
    api: *mut *const std::ffi::c_char,
    is_floating: *mut bool,
) -> bool {
    unsafe { *is_floating = false };
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
}

pub unsafe extern "C" fn create(_plugin: *const clap_plugin_t, _api: *const std::ffi::c_char, _is_floating: bool) -> bool {
    true
}

pub unsafe extern "C" fn destroy(plugin: *const clap_plugin_t) {
    let p = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };
    p.gui.window = None;
}

pub unsafe extern "C" fn set_scale(_plugin: *const clap_plugin_t, _scale: f64) -> bool {
    false
}

pub unsafe extern "C" fn get_size(plugin: *const clap_plugin_t, width: *mut u32, height: *mut u32) -> bool {
    let p = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };
    unsafe { *width = p.gui.width };
    unsafe { *height = p.gui.height };
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
    let p = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };
    p.gui.width = width;
    p.gui.height = height;
    if let Some(wv) = &p.gui.window {
        let _ = wv.set_bounds(Rect {
            position: LogicalPosition::new(0u32, 0u32).into(),
            size: LogicalSize::new(width, height).into(),
        });
    }
    true
}

pub unsafe extern "C" fn set_parent(plugin: *const clap_plugin_t, window: *const clap_window_t) -> bool {
    let plugin_ref = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };

    let params_rx: Arc<ArcSwap<PluginParameters>> = Arc::clone(&plugin_ref.parameters_rx);
    let params_wx: Arc<Mutex<PluginParameters>> = Arc::clone(&plugin_ref.parameters_wx);

    let parent = match unsafe { host_window_from_clap(window) } {
        Some(host_window) => host_window,
        None => return false,
    };

    let builder = WebViewBuilder::new()
        .with_html(GUI_HTML)
        .with_bounds(Rect {
            position: LogicalPosition::new(0u32, 0u32).into(),
            size: LogicalSize::new(plugin_ref.gui.width, plugin_ref.gui.height).into(),
        })
        .with_ipc_handler(move |request| {
            handle_ui_event(&params_rx, &params_wx, request.body());
        });

    if let Ok(webview) = builder.build_as_child(&parent) {
        if let Ok(mut queue) = plugin_ref.gui.message_queue.lock() {
            queue.push(GUIRequest::ParamValueBatch(
                plugin_ref
                    .parameters_rx
                    .load()
                    .main_thread_parameters
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (index, *value as f64))
                    .collect::<Vec<(usize, f64)>>(),
            ));

            for msg in queue.drain(..) {
                send_ui_event(&webview, msg);
            }
        }

        plugin_ref.gui.window = Some(webview);

        return true;
    }

    false
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
