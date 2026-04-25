use crate::clap::*;

pub fn clap_version_is_compatible(v: clap_version) -> bool {
    v.major >= 1
}

pub const CLAP_VERSION_INIT: clap_version = clap_version {
    major: 1,
    minor: 2,
    revision: 7,
};
