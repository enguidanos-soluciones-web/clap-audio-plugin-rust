#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("nam.h");

        type NamDsp;

        fn load(json: &str) -> UniquePtr<NamDsp>;
        fn process(dsp: Pin<&mut NamDsp>, input: &[f64], output: &mut [f64]);
        fn reset(dsp: Pin<&mut NamDsp>, sample_rate: f64, max_block_size: i32);
        fn reset_and_prewarm(dsp: Pin<&mut NamDsp>, sample_rate: f64, max_block_size: i32);
        fn get_sample_rate_from_nam_file(json: &str) -> f64;
        fn has_loudness(dsp: &NamDsp) -> bool;
        fn get_loudness(dsp: &NamDsp) -> f64;
    }
}
