use crate::{
    clap::*,
    state::{AudioThreadState, ParamEvent},
};

pub fn handle_clap_event(audio_thread: &mut AudioThreadState, event: *const clap_event_header_t) {
    audio_thread.assert_audio_thread();

    let event_ref = unsafe { event.as_ref_unchecked() };
    if event_ref.space_id != CLAP_CORE_EVENT_SPACE_ID {
        return;
    }

    if event_ref.type_ as u32 == CLAP_EVENT_PARAM_VALUE {
        let value_event = unsafe { (event as *const clap_event_param_value_t).as_ref_unchecked() };
        let id = value_event.param_id as usize;
        let value = value_event.value as f32;

        let _ = audio_thread.daw_events.push(ParamEvent::Automation { id, value });

        // Request host call on_main_thread to update the snapshot
        unsafe {
            if let Some(request_callback) = (*audio_thread.host).request_callback {
                request_callback(audio_thread.host);
            }
        }
    }
}
