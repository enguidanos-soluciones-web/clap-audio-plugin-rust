use crate::{clap::*, plugin::Plugin};
use std::sync::Arc;

pub unsafe extern "C" fn handle_clap_event(plugin: &mut Plugin, event: *const clap_event_header_t) {
    let event_ref = unsafe { event.as_ref_unchecked() };
    if event_ref.space_id != CLAP_CORE_EVENT_SPACE_ID {
        return;
    }

    if event_ref.type_ as u32 == CLAP_EVENT_PARAM_VALUE {
        let value_event = event as *const clap_event_param_value_t;
        let value_event_ref = unsafe { value_event.as_ref_unchecked() };

        if let Ok(mut params) = plugin.state.parameters_wx.try_lock() {
            params.audio_thread_parameters[value_event_ref.param_id as usize] = value_event_ref.value as f32;
            params.audio_thread_parameters_changed[value_event_ref.param_id as usize] = true;
            plugin.state.parameters_rx.store(Arc::new(*params));
        }
    }
}
