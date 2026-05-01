use super::{Action, load_model::LoadModel};
use crate::state::GuiRequest;

pub enum AnyAction {
    LoadModel(LoadModel),
}

impl AnyAction {
    pub fn on_click(&self) -> GuiRequest {
        match self {
            AnyAction::LoadModel(a) => a.on_click(),
        }
    }
}

impl TryFrom<usize> for AnyAction {
    type Error = ();

    fn try_from(id: usize) -> Result<Self, Self::Error> {
        match id {
            LoadModel::ID => Ok(AnyAction::LoadModel(LoadModel)),
            _ => Err(()),
        }
    }
}
