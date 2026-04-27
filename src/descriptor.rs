use std::ffi::c_char;

use crate::{clap::*, version::CLAP_VERSION_INIT};

unsafe impl Sync for clap_plugin_descriptor_t {}

struct FeatureList([*const c_char; 3]);

unsafe impl Sync for FeatureList {}

static PLUGIN_FEATURES: FeatureList = FeatureList([
    CLAP_PLUGIN_FEATURE_MONO.as_ptr(),
    CLAP_PLUGIN_FEATURE_AUDIO_EFFECT.as_ptr(),
    std::ptr::null(),
]);

pub static PLUGIN_DESCRIPTOR: clap_plugin_descriptor_t = clap_plugin_descriptor {
    clap_version: CLAP_VERSION_INIT,
    id: c"com.enguidanosweb.Marshallian".as_ptr(),
    name: c"Marshallian".as_ptr(),
    vendor: c"enguidanosweb".as_ptr(),
    url: c"https://enguidanosweb.com".as_ptr(),
    manual_url: c"https://enguidanosweb.com".as_ptr(),
    support_url: c"https://enguidanosweb.com".as_ptr(),
    version: c"0.0.1".as_ptr(),
    description: c"Marshallian".as_ptr(),
    features: PLUGIN_FEATURES.0.as_ptr(),
};
