#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("nam.h");

        fn activation_enable_fast_tanh();

        type NamDsp;

        fn dsp_load(json: &str) -> UniquePtr<NamDsp>;
        fn dsp_process(dsp: Pin<&mut NamDsp>, input: &[f64], output: &mut [f64]);
        fn dsp_reset(dsp: Pin<&mut NamDsp>, sample_rate: f64, max_block_size: i32);
        fn get_sample_rate_from_nam_file(json: &str) -> f64;
    }
}
