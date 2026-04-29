use crate::{
    dsp::nam,
    helper::{DecibelConversion, db_to_linear},
    parameters::{Parameter, Range, input_gain::InputGain, output_gain::OutputGain, tone::Tone},
    state::AudioThreadState,
};

pub fn render_audio_f64(audio_thread: &mut AudioThreadState, input: *const f64, output: *mut f64, nframes: usize) {
    audio_thread.assert_audio_thread();

    let snapshot = audio_thread.param_snapshot.load();

    let input_gain = db_to_linear(snapshot.values[Parameter::<InputGain, Range>::ID], DecibelConversion::Amplitude);
    let output_gain = db_to_linear(snapshot.values[Parameter::<OutputGain, Range>::ID], DecibelConversion::Amplitude);

    let input_slice = unsafe { std::slice::from_raw_parts(input, nframes) };
    let output_slice = unsafe { std::slice::from_raw_parts_mut(output, nframes) };

    // 1. Apply input gain
    for i in 0..nframes {
        audio_thread.input_buf[i] = input_slice[i] * input_gain;
    }

    if let Some(nam_model) = audio_thread.nam_model.as_mut() {
        // 2. Process with NAM
        nam::ffi::process(
            nam_model.pin_mut(),
            &audio_thread.input_buf[..nframes],
            &mut audio_thread.output_buf[..nframes],
        );

        let tone = snapshot.values[Parameter::<Tone, Range>::ID];
        let cutoff = Parameter::<Tone, Range>::to_hertz(tone);
        audio_thread.lowpass_filter.set_cutoff(cutoff, audio_thread.sample_rate);

        // 3. DC Filter, Loudness Correction and output gain
        for i in 0..nframes {
            let dc_filter = audio_thread.dc_filter.process_sample(audio_thread.output_buf[i]);
            let lowpass_filter = audio_thread.lowpass_filter.process_sample(dc_filter);
            let sample = lowpass_filter * audio_thread.nam_loudness_correction * output_gain;
            output_slice[i] = sample;
        }
    } else {
        output_slice.copy_from_slice(input_slice);
    }
}

pub fn render_audio_f32(audio_thread: &mut AudioThreadState, input: *const f32, output: *mut f32, nframes: usize) {
    audio_thread.assert_audio_thread();

    let snapshot = audio_thread.param_snapshot.load();

    let input_gain = db_to_linear(snapshot.values[Parameter::<InputGain, Range>::ID], DecibelConversion::Amplitude);
    let output_gain = db_to_linear(snapshot.values[Parameter::<OutputGain, Range>::ID], DecibelConversion::Amplitude);

    let input_slice = unsafe { std::slice::from_raw_parts(input, nframes) };
    let output_slice = unsafe { std::slice::from_raw_parts_mut(output, nframes) };

    // 1. Apply input gain
    for i in 0..nframes {
        audio_thread.input_buf[i] = input_slice[i] as f64 * input_gain;
    }

    if let Some(nam_model) = audio_thread.nam_model.as_mut() {
        // 2. Process with NAM
        nam::ffi::process(
            nam_model.pin_mut(),
            &audio_thread.input_buf[..nframes],
            &mut audio_thread.output_buf[..nframes],
        );

        let tone = snapshot.values[Parameter::<Tone, Range>::ID];
        let tone_cutoff = Parameter::<Tone, Range>::to_hertz(tone);
        audio_thread.lowpass_filter.set_cutoff(tone_cutoff, audio_thread.sample_rate);

        // 3. DC Filter, Loudness Correction and output gain
        for i in 0..nframes {
            let dc_filter = audio_thread.dc_filter.process_sample(audio_thread.output_buf[i]);
            let lowpass_filter = audio_thread.lowpass_filter.process_sample(dc_filter);
            let sample = lowpass_filter * audio_thread.nam_loudness_correction * output_gain;
            output_slice[i] = sample as f32;
        }
    } else {
        output_slice.copy_from_slice(input_slice);
    }
}
