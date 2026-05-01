use crate::state::GuiRequest;
use std::sync::Arc;

pub type Dispatcher = Arc<dyn Fn(GuiRequest) + Send + Sync>;
