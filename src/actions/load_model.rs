use crate::{actions::Action, state::GuiRequest};

pub struct LoadModel;

impl LoadModel {
    pub const ID: usize = 0;
}

impl Action for LoadModel {
    fn dom_id(&self) -> &'static str {
        "load-model"
    }

    fn on_click(&self) -> GuiRequest {
        GuiRequest::OpenFileBrowser
    }
}
