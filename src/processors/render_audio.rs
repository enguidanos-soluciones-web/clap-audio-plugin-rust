use crate::{
    dsp::nam,
    helper::{DecibelConversion, db_to_linear},
    parameters::{Parameter, Range, blend::Blend, input_gain::InputGain, output_gain::OutputGain, tone::Tone},
    state::AudioThreadState,
};

fn apply_pending_model_update(audio_thread: &mut AudioThreadState) {
    if let Some(update) = audio_thread.model_updates.pop() {
        audio_thread.nam_model = Some(update.model);
        audio_thread.nam_loudness_correction = update.loudness_correction;
        audio_thread.dc_filter.reset();
        audio_thread.klon_buffer.reset();
        audio_thread.lowpass_filter.reset();
    }
}

pub fn render_audio_f64(audio_thread: &mut AudioThreadState, input: *const f64, output: *mut f64, nframes: usize) {
    audio_thread.assert_audio_thread();
    apply_pending_model_update(audio_thread);

    let snapshot = audio_thread.param_snapshot.load();

    let input_gain = db_to_linear(snapshot.values[Parameter::<InputGain, Range>::ID], DecibelConversion::Amplitude);
    let output_gain = db_to_linear(snapshot.values[Parameter::<OutputGain, Range>::ID], DecibelConversion::Amplitude);
    let blend = snapshot.values[Parameter::<Blend, Range>::ID];

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

        // 3. DC filter, loudness correction, output gain, dry/wet blend, then tone lowpass.
        // dry: raw input signal — no input gain, no NAM, no loudness correction.
        // wet: NAM → DC filter → loudness correction → output gain.
        // The tone lowpass is applied to the blended result so it shapes both paths equally.
        for i in 0..nframes {
            let dc_filtered = audio_thread.dc_filter.process_sample(audio_thread.output_buf[i]);
            let wet = dc_filtered * audio_thread.nam_loudness_correction * output_gain;
            let dry = audio_thread.klon_buffer.process_sample(input_slice[i]);
            let blended = Parameter::<Blend, Range>::mix(dry, wet, blend);
            output_slice[i] = audio_thread.lowpass_filter.process_sample(blended);
        }
    } else {
        output_slice.copy_from_slice(input_slice);
    }
}

pub fn render_audio_f32(audio_thread: &mut AudioThreadState, input: *const f32, output: *mut f32, nframes: usize) {
    audio_thread.assert_audio_thread();
    apply_pending_model_update(audio_thread);

    let snapshot = audio_thread.param_snapshot.load();

    let input_gain = db_to_linear(snapshot.values[Parameter::<InputGain, Range>::ID], DecibelConversion::Amplitude);
    let output_gain = db_to_linear(snapshot.values[Parameter::<OutputGain, Range>::ID], DecibelConversion::Amplitude);
    let blend = snapshot.values[Parameter::<Blend, Range>::ID];

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

        // 3. DC filter, loudness correction, output gain, dry/wet blend, then tone lowpass.
        // dry: raw input signal — no input gain, no NAM, no loudness correction.
        // wet: NAM → DC filter → loudness correction → output gain.
        // The tone lowpass is applied to the blended result so it shapes both paths equally.
        for i in 0..nframes {
            let dc_filtered = audio_thread.dc_filter.process_sample(audio_thread.output_buf[i]);
            let wet = dc_filtered * audio_thread.nam_loudness_correction * output_gain;
            let dry = audio_thread.klon_buffer.process_sample(input_slice[i] as f64);
            let blended = Parameter::<Blend, Range>::mix(dry, wet, blend);
            output_slice[i] = audio_thread.lowpass_filter.process_sample(blended) as f32;
        }
    } else {
        output_slice.copy_from_slice(input_slice);
    }
}
