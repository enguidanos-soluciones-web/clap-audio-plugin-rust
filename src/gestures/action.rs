use crate::{actions::any::AnyAction, state::GuiRequest};

pub struct ActiveAction(AnyAction);

impl ActiveAction {
    pub fn from_index(index: usize) -> Option<Self> {
        AnyAction::try_from(index).ok().map(ActiveAction)
    }

    pub fn on_click(&self) -> GuiRequest {
        self.0.on_click()
    }
}
