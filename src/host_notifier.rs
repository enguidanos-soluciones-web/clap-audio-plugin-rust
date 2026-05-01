use std::sync::Arc;

use crate::clap::clap_host_t;

/// Wraps the CLAP host pointer and exposes a safe `notify()` method.
///
/// The raw pointer is valid for the entire plugin lifetime — the host
/// guarantees this. `request_callback` is thread-safe per the CLAP spec,
/// so `Send + Sync` are sound.
pub struct HostNotifier(*const clap_host_t);

// SAFETY: CLAP spec guarantees `request_callback` may be called from any thread.
// The host pointer is valid for the plugin lifetime.
unsafe impl Send for HostNotifier {}
unsafe impl Sync for HostNotifier {}

impl HostNotifier {
    pub fn new(host: *const clap_host_t) -> Arc<Self> {
        Arc::new(Self(host))
    }

    pub fn notify(&self) {
        unsafe {
            if let Some(request_callback) = (*self.0).request_callback {
                request_callback(self.0);
            }
        }
    }
}
