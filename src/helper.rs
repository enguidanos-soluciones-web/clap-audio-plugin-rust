use std::ffi::c_char;

pub enum DecibelConversion {
    Amplitude,
    #[allow(dead_code)]
    Power,
}

pub fn db_to_linear(db: f64, conv: DecibelConversion) -> f64 {
    f64::powf(
        10.0,
        db / match conv {
            DecibelConversion::Amplitude => 20.0,
            DecibelConversion::Power => 10.0,
        },
    )
}

pub fn copy_cstr(dst: &mut [c_char], src: &[u8]) {
    let len = src.len().min(dst.len() - 1);
    for (d, s) in dst[..len].iter_mut().zip(src[..len].iter()) {
        *d = *s as c_char;
    }
    dst[len] = 0; // null terminator
}
