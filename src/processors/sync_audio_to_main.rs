use crate::parameters::any::PARAMS_COUNT;
use crate::plugin::Plugin;
use std::sync::Arc;

pub unsafe extern "C" fn sync_audio_to_main(plugin: &mut Plugin) -> bool {
    let mut anychanged = false;

    if let Ok(mut params) = plugin.parameters_wx.try_lock() {
        for n in 0..PARAMS_COUNT {
            if params.audio_thread_parameters_changed[n] {
                params.audio_thread_parameters_changed[n] = false;
                params.main_thread_parameters[n] = params.audio_thread_parameters[n];

                anychanged = true;
            }
        }

        if anychanged {
            plugin.parameters_rx.store(Arc::new(*params));
        }
    }

    anychanged
}
