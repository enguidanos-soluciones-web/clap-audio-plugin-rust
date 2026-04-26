use crate::{
    helper::db_to_linear,
    nam,
    parameters::{
        base::{Parameter, Range},
        input_gain::InputGain,
        output_gain::OutputGain,
    },
    plugin::Plugin,
};

pub unsafe fn render_audio(plugin: &mut Plugin, input: *const f32, output: *mut f32, nframes: usize) {
    let params = plugin.parameters_rx.load();

    let input_gain = db_to_linear(params.audio_thread_parameters[Parameter::<InputGain, Range>::ID] as f64);
    let output_gain = db_to_linear(params.audio_thread_parameters[Parameter::<OutputGain, Range>::ID] as f64);

    let input_slice = unsafe { std::slice::from_raw_parts(input, nframes) };
    let output_slice = unsafe { std::slice::from_raw_parts_mut(output, nframes) };

    for i in 0..nframes {
        plugin.input_buf[i] = input_slice[i] as f64 * input_gain;
    }

    if let Some(model) = plugin.model.as_mut() {
        nam::ffi::dsp_process(model.pin_mut(), &plugin.input_buf[..nframes], &mut plugin.output_buf[..nframes]);

        for i in 0..nframes {
            output_slice[i] = (plugin.output_buf[i] * output_gain) as f32;
        }
    } else {
        output_slice.copy_from_slice(input_slice);
    }
}
