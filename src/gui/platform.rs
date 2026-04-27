use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle};

pub struct ClapParentWindow {
    pub raw_window: RawWindowHandle,
    pub raw_display: RawDisplayHandle,
}

unsafe impl HasRawWindowHandle for ClapParentWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.raw_window
    }
}

unsafe impl HasRawDisplayHandle for ClapParentWindow {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.raw_display
    }
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGColorCreateSRGB(red: f64, green: f64, blue: f64, alpha: f64) -> *mut std::ffi::c_void;
    fn CGColorRelease(color: *mut std::ffi::c_void);
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
pub fn set_window_background_color(window: &baseview::Window) {
    use objc::{msg_send, sel, sel_impl, runtime::Object};

    let RawWindowHandle::AppKit(handle) = window.raw_window_handle() else { return };

    unsafe {
        let ns_view = handle.ns_view as *mut Object;

        let _: () = msg_send![ns_view, setWantsLayer: true];

        let layer: *mut Object = msg_send![ns_view, layer];
        if layer.is_null() { return; }

        // #1c1c1e → rgb(28, 28, 30)
        let color = CGColorCreateSRGB(28.0 / 255.0, 28.0 / 255.0, 30.0 / 255.0, 1.0);
        let _: () = msg_send![layer, setBackgroundColor: color];
        CGColorRelease(color);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn set_window_background_color(_window: &baseview::Window) {}

#[cfg(target_os = "macos")]
pub unsafe fn make_parent_window(window: *const crate::clap::clap_window_t) -> ClapParentWindow {
    use raw_window_handle::{AppKitDisplayHandle, AppKitWindowHandle};
    let ns_view = unsafe { (*window).__bindgen_anon_1.cocoa };
    let mut handle = AppKitWindowHandle::empty();
    handle.ns_view = ns_view;
    ClapParentWindow {
        raw_window: RawWindowHandle::AppKit(handle),
        raw_display: RawDisplayHandle::AppKit(AppKitDisplayHandle::empty()),
    }
}

#[cfg(target_os = "linux")]
pub unsafe fn make_parent_window(window: *const crate::clap::clap_window_t) -> ClapParentWindow {
    use raw_window_handle::{XlibDisplayHandle, XlibWindowHandle};
    let xid = unsafe { (*window).__bindgen_anon_1.x11 };
    let mut handle = XlibWindowHandle::empty();
    handle.window = xid as _;
    ClapParentWindow {
        raw_window: RawWindowHandle::Xlib(handle),
        raw_display: RawDisplayHandle::Xlib(XlibDisplayHandle::empty()),
    }
}

#[cfg(target_os = "windows")]
pub unsafe fn make_parent_window(window: *const crate::clap::clap_window_t) -> ClapParentWindow {
    use raw_window_handle::{Win32WindowHandle, WindowsDisplayHandle};
    let hwnd = unsafe { (*window).__bindgen_anon_1.win32 };
    let mut handle = Win32WindowHandle::empty();
    handle.hwnd = hwnd as *mut _;
    ClapParentWindow {
        raw_window: RawWindowHandle::Win32(handle),
        raw_display: RawDisplayHandle::Windows(WindowsDisplayHandle::empty()),
    }
}

pub fn to_wgpu_window_handle(h: RawWindowHandle) -> wgpu::rwh::RawWindowHandle {
    use wgpu::rwh as rwh6;
    match h {
        #[cfg(target_os = "macos")]
        RawWindowHandle::AppKit(h) => {
            let ns_view = std::ptr::NonNull::new(h.ns_view).expect("null NSView");
            rwh6::RawWindowHandle::AppKit(rwh6::AppKitWindowHandle::new(ns_view))
        }
        #[cfg(target_os = "linux")]
        RawWindowHandle::Xlib(h) => rwh6::RawWindowHandle::Xlib(rwh6::XlibWindowHandle::new(h.window)),
        #[cfg(target_os = "windows")]
        RawWindowHandle::Win32(h) => {
            use std::num::NonZeroIsize;
            let hwnd = NonZeroIsize::new(h.hwnd as isize).expect("null HWND");
            rwh6::RawWindowHandle::Win32(rwh6::Win32WindowHandle::new(hwnd))
        }
        _ => panic!("unsupported window handle type"),
    }
}

pub fn to_wgpu_display_handle(h: RawDisplayHandle) -> wgpu::rwh::RawDisplayHandle {
    use wgpu::rwh as rwh6;
    match h {
        #[cfg(target_os = "macos")]
        RawDisplayHandle::AppKit(_) => rwh6::RawDisplayHandle::AppKit(rwh6::AppKitDisplayHandle::new()),
        #[cfg(target_os = "linux")]
        RawDisplayHandle::Xlib(h) => {
            let display = std::ptr::NonNull::new(h.display);
            rwh6::RawDisplayHandle::Xlib(rwh6::XlibDisplayHandle::new(display, h.screen))
        }
        #[cfg(target_os = "windows")]
        RawDisplayHandle::Windows(_) => rwh6::RawDisplayHandle::Windows(rwh6::WindowsDisplayHandle::new()),
        _ => panic!("unsupported display handle type"),
    }
}
