use crate::{parameters::any::PARAMS_COUNT, plugin::PluginParameters};
use arc_swap::ArcSwap;
use serde::Deserialize;
use serde_json::from_str;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
pub struct GUIReply {
    pub id: u64,
    pub value: f64,
}

pub fn handle_ui_event(params_rx: &Arc<ArcSwap<PluginParameters>>, params_wx: &Arc<Mutex<PluginParameters>>, reply: &str) {
    let Ok(reply) = from_str::<GUIReply>(reply) else {
        return;
    };

    if reply.id as usize >= PARAMS_COUNT {
        return;
    }

    if let Ok(mut params) = params_wx.lock() {
        params.main_thread_parameters[reply.id as usize] = reply.value as f32;
        params.main_thread_parameters_changed[reply.id as usize] = true;
        params_rx.store(Arc::new(*params));
    }
}
