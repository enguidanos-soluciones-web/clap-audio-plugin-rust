use crate::{
    gui::widget::Widget,
    parameters::{Action, PARAMETER_GESTURE_SINGLE_CLICK, Parameter},
    state::GuiRequest,
};

#[derive(Clone, Copy)]
pub struct LoadModel;

impl Parameter<LoadModel, Action> {
    pub const ID: usize = 4;

    pub fn new() -> Self {
        Self {
            id: Self::ID,
            name: "Load Model",
            gestures: PARAMETER_GESTURE_SINGLE_CLICK,
            behave: Action,
            _marker_type: std::marker::PhantomData,
            _marker_behaviour: std::marker::PhantomData,
        }
    }

    pub fn on_single_click(&self) -> GuiRequest {
        GuiRequest::OpenFileBrowser
    }
}

impl Widget for Parameter<LoadModel, Action> {
    fn element_id(&self) -> &'static str {
        "load-model"
    }

    fn param_id(&self) -> usize {
        Self::ID
    }
}
