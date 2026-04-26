use std::sync::{Arc, Mutex};

use raw_window_handle::{HasWindowHandle, RawWindowHandle, WindowHandle};
use wry::{Rect, WebViewBuilder, dpi::{LogicalPosition, LogicalSize}};

use crate::{
    clap::*,
    parameters::any::PARAMS_COUNT,
    plugin::{Plugin, PluginParameters},
};

const GUI_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  background: #1a1a2e;
  color: #e0e0e0;
  font-family: 'Courier New', monospace;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  gap: 2rem;
  user-select: none;
}
h1 { font-size: 1em; color: #7aa2f7; letter-spacing: 0.3em; }
.params { display: flex; gap: 4rem; }
.param { display: flex; flex-direction: column; align-items: center; gap: 0.8rem; }
.param-label { font-size: 0.7em; color: #9aa5ce; text-transform: uppercase; letter-spacing: 0.12em; }
.param-value { font-size: 0.95em; color: #c0caf5; min-width: 7ch; text-align: center; }
input[type=range] {
  -webkit-appearance: none;
  width: 180px;
  height: 4px;
  background: #364a82;
  border-radius: 2px;
  outline: none;
  cursor: pointer;
}
input[type=range]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  background: #7aa2f7;
  border-radius: 50%;
  cursor: pointer;
}
input[type=range]::-webkit-slider-thumb:hover { background: #89b4fa; }
</style>
</head>
<body>
<h1>NAM PLAYER</h1>
<div class="params">
  <div class="param">
    <span class="param-label">Input Gain</span>
    <input type="range" id="input_gain" min="-20" max="20" step="0.1" value="0">
    <span class="param-value" id="input_gain_val">+0.0 dB</span>
  </div>
  <div class="param">
    <span class="param-label">Output Gain</span>
    <input type="range" id="output_gain" min="-20" max="20" step="0.1" value="0">
    <span class="param-value" id="output_gain_val">+0.0 dB</span>
  </div>
</div>
<script>
var PARAM_IDS = ['input_gain', 'output_gain'];

function fmt(v) {
  var n = parseFloat(v);
  return (n >= 0 ? '+' : '') + n.toFixed(1) + ' dB';
}

PARAM_IDS.forEach(function(id, idx) {
  var el = document.getElementById(id);
  var val = document.getElementById(id + '_val');
  el.addEventListener('input', function() {
    val.textContent = fmt(el.value);
    window.ipc.postMessage(JSON.stringify({ id: idx, value: parseFloat(el.value) }));
  });
});

window.__setParam = function(idx, value) {
  if (idx >= 0 && idx < PARAM_IDS.length) {
    var id = PARAM_IDS[idx];
    var el = document.getElementById(id);
    var val = document.getElementById(id + '_val');
    if (el) {
      el.value = value;
      val.textContent = fmt(value);
    }
  }
};
</script>
</body>
</html>"#;

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

pub unsafe extern "C" fn is_api_supported(
    _plugin: *const clap_plugin_t,
    api: *const std::ffi::c_char,
    is_floating: bool,
) -> bool {
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

pub unsafe extern "C" fn create(
    _plugin: *const clap_plugin_t,
    _api: *const std::ffi::c_char,
    _is_floating: bool,
) -> bool {
    true
}

pub unsafe extern "C" fn destroy(plugin: *const clap_plugin_t) {
    let p = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };
    p.gui_window = None;
}

pub unsafe extern "C" fn set_scale(_plugin: *const clap_plugin_t, _scale: f64) -> bool {
    false
}

pub unsafe extern "C" fn get_size(
    plugin: *const clap_plugin_t,
    width: *mut u32,
    height: *mut u32,
) -> bool {
    let p = unsafe { ((*plugin).plugin_data as *const Plugin).as_ref_unchecked() };
    unsafe { *width = p.gui_width };
    unsafe { *height = p.gui_height };
    true
}

pub unsafe extern "C" fn can_resize(_plugin: *const clap_plugin_t) -> bool {
    true
}

pub unsafe extern "C" fn get_resize_hints(
    _plugin: *const clap_plugin_t,
    hints: *mut clap_gui_resize_hints_t,
) -> bool {
    let h = unsafe { hints.as_mut_unchecked() };
    h.can_resize_horizontally = true;
    h.can_resize_vertically = true;
    h.preserve_aspect_ratio = false;
    h.aspect_ratio_width = 0;
    h.aspect_ratio_height = 0;
    true
}

pub unsafe extern "C" fn adjust_size(
    _plugin: *const clap_plugin_t,
    _width: *mut u32,
    _height: *mut u32,
) -> bool {
    true
}

pub unsafe extern "C" fn set_size(plugin: *const clap_plugin_t, width: u32, height: u32) -> bool {
    let p = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };
    p.gui_width = width;
    p.gui_height = height;
    if let Some(wv) = &p.gui_window {
        let _ = wv.set_bounds(Rect {
            position: LogicalPosition::new(0u32, 0u32).into(),
            size: LogicalSize::new(width, height).into(),
        });
    }
    true
}

pub unsafe extern "C" fn set_parent(
    plugin: *const clap_plugin_t,
    window: *const clap_window_t,
) -> bool {
    let p = unsafe { ((*plugin).plugin_data as *mut Plugin).as_mut_unchecked() };
    let params_wx: Arc<Mutex<PluginParameters>> = Arc::clone(&p.parameters_wx);

    let builder = WebViewBuilder::new()
        .with_html(GUI_HTML)
        .with_bounds(Rect {
            position: LogicalPosition::new(0u32, 0u32).into(),
            size: LogicalSize::new(p.gui_width, p.gui_height).into(),
        })
        .with_ipc_handler(move |request| {
            let body = request.body();
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(body) {
                if let (Some(id), Some(value)) = (
                    msg.get("id").and_then(|v| v.as_u64()),
                    msg.get("value").and_then(|v| v.as_f64()),
                ) {
                    if let Ok(mut params) = params_wx.lock() {
                        let idx = id as usize;
                        if idx < PARAMS_COUNT {
                            params.main_thread_parameters[idx] = value as f32;
                            params.main_thread_parameters_changed[idx] = true;
                        }
                    }
                }
            }
        });

    #[cfg(target_os = "windows")]
    {
        use raw_window_handle::Win32WindowHandle;

        struct HostHwnd(std::num::NonZeroIsize);
        impl HasWindowHandle for HostHwnd {
            fn window_handle(
                &self,
            ) -> Result<WindowHandle<'_>, raw_window_handle::HandleError> {
                Ok(unsafe {
                    WindowHandle::borrow_raw(RawWindowHandle::Win32(Win32WindowHandle::new(
                        self.0,
                    )))
                })
            }
        }

        let hwnd = unsafe { (*window).__bindgen_anon_1.win32 } as isize;
        let parent = match std::num::NonZeroIsize::new(hwnd) {
            Some(n) => HostHwnd(n),
            None => return false,
        };
        return match builder.build_as_child(&parent) {
            Ok(wv) => { p.gui_window = Some(wv); true }
            Err(_) => false,
        };
    }

    #[cfg(target_os = "macos")]
    {
        use raw_window_handle::AppKitWindowHandle;

        struct HostNsView(std::ptr::NonNull<std::ffi::c_void>);
        impl HasWindowHandle for HostNsView {
            fn window_handle(
                &self,
            ) -> Result<WindowHandle<'_>, raw_window_handle::HandleError> {
                Ok(unsafe {
                    WindowHandle::borrow_raw(RawWindowHandle::AppKit(AppKitWindowHandle::new(
                        self.0,
                    )))
                })
            }
        }

        let cocoa = unsafe { (*window).__bindgen_anon_1.cocoa };
        let parent = match std::ptr::NonNull::new(cocoa as *mut std::ffi::c_void) {
            Some(nn) => HostNsView(nn),
            None => return false,
        };
        return match builder.build_as_child(&parent) {
            Ok(wv) => { p.gui_window = Some(wv); true }
            Err(_) => false,
        };
    }
}

pub unsafe extern "C" fn set_transient(
    _plugin: *const clap_plugin_t,
    _window: *const clap_window_t,
) -> bool {
    false
}

pub unsafe extern "C" fn suggest_title(
    _plugin: *const clap_plugin_t,
    _title: *const std::ffi::c_char,
) {
}

pub unsafe extern "C" fn show(_plugin: *const clap_plugin_t) -> bool {
    true
}

pub unsafe extern "C" fn hide(_plugin: *const clap_plugin_t) -> bool {
    true
}
