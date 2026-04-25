use crate::{clap::*, gui::parameters::any::PARAMS_COUNT, plugin::Plugin};
use std::sync::Arc;

pub unsafe extern "C" fn sync_main_to_audio(plugin: &mut Plugin, out: *const clap_output_events_t) {
    if let Ok(mut params) = plugin.parameters_wx.try_lock() {
        let mut anychanged = false;

        for n in 0..PARAMS_COUNT {
            if params.main_thread_parameters_changed[n] {
                let mut event = unsafe { std::mem::zeroed::<clap_event_param_value_t>() };
                event.header.size = std::mem::size_of::<clap_event_param_value_t>() as u32;
                event.header.time = 0;
                event.header.space_id = CLAP_CORE_EVENT_SPACE_ID;
                event.header.type_ = CLAP_EVENT_PARAM_VALUE as u16;
                event.header.flags = 0;
                event.param_id = n as u32;
                event.cookie = std::ptr::null_mut();
                event.note_id = -1;
                event.port_index = -1;
                event.channel = -1;
                event.key = -1;
                // must be main_thread (new value), not audio_thread (old value) — host automation records this
                event.value = params.main_thread_parameters[n] as f64;

                let out_ref = unsafe { out.as_ref_unchecked() };

                if let Some(try_push) = out_ref.try_push {
                    if unsafe { try_push(out, &event.header) } {
                        params.main_thread_parameters_changed[n] = false;
                        anychanged = true;
                    } else {
                        params.main_thread_parameters_changed[n] = true;
                    }

                    params.audio_thread_parameters[n] = params.main_thread_parameters[n];
                }
            }
        }

        if anychanged {
            plugin.parameters_rx.store(Arc::new(*params));
        }
    }
}
