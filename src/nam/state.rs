use crate::nam::nam::ffi::NamDsp;

pub struct NAMState {
    pub model: Option<cxx::UniquePtr<NamDsp>>,
}
