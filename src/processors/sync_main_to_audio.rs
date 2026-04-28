use crate::{
    clap::*,
    state::{AudioThreadState, ParamEvent},
};

pub fn sync_main_to_audio(audio_thread: &mut AudioThreadState, out: *const clap_output_events_t) {
    audio_thread.assert_audio_thread();

    let out_ref = unsafe { out.as_ref_unchecked() };
    let Some(try_push) = out_ref.try_push else { return };

    while let Ok(change) = audio_thread.param_changes_rx.pop() {
        let mut event = unsafe { std::mem::zeroed::<clap_event_param_value_t>() };
        event.header.size = std::mem::size_of::<clap_event_param_value_t>() as u32;
        event.header.time = 0;
        event.header.space_id = CLAP_CORE_EVENT_SPACE_ID;
        event.header.type_ = CLAP_EVENT_PARAM_VALUE as u16;
        event.header.flags = 0;
        event.param_id = change.id as u32;
        event.cookie = std::ptr::null_mut();
        event.note_id = -1;
        event.port_index = -1;
        event.channel = -1;
        event.key = -1;
        event.value = change.value as f64;

        if unsafe { try_push(out, &event.header) } {
            // Host accepted the change. We need to ack the man-thread.
            let _ = audio_thread.daw_events_tx.push(ParamEvent::Ack);
        } else {
            // Host not accepted the change. Main-thread preserves changed=true and will requeue on the next attempt.
            let _ = audio_thread.daw_events_tx.push(ParamEvent::Nack { id: change.id });
        }
    }
}
