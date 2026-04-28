use crate::{
    dsp::nam,
    helper::db_to_linear,
    parameters::{Parameter, Range, input_gain::InputGain, output_gain::OutputGain},
    state::AudioThreadState,
};

pub fn render_audio(audio_thread: &mut AudioThreadState, input: *const f32, output: *mut f32, nframes: usize) {
    audio_thread.assert_audio_thread();

    let snapshot = audio_thread.param_snapshot.load();

    let input_gain = db_to_linear(snapshot.values[Parameter::<InputGain, Range>::ID] as f64);
    let output_gain = db_to_linear(snapshot.values[Parameter::<OutputGain, Range>::ID] as f64);

    let input_slice = unsafe { std::slice::from_raw_parts(input, nframes) };
    let output_slice = unsafe { std::slice::from_raw_parts_mut(output, nframes) };

    // 1. Apply input gain
    for i in 0..nframes {
        audio_thread.input_buf[i] = input_slice[i] as f64 * input_gain;
    }

    if let Some(nam_model) = audio_thread.nam_model.as_mut() {
        // 2. Process with NAM
        nam::ffi::dsp_process(
            nam_model.pin_mut(),
            &audio_thread.input_buf[..nframes],
            &mut audio_thread.output_buf[..nframes],
        );

        // 3. DC Filter and output gain
        for i in 0..nframes {
            let dc_filtered_sample = audio_thread.dc_filter.process_sample(audio_thread.output_buf[i]);
            output_slice[i] = (dc_filtered_sample * output_gain) as f32;
        }
    } else {
        output_slice.copy_from_slice(input_slice);
    }
}
