pub mod any;
pub mod load_model;

use crate::state::GuiRequest;

pub trait Action {
    fn dom_id(&self) -> &'static str;
    fn on_click(&self) -> GuiRequest;
}
