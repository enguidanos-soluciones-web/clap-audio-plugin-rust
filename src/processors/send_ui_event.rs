use wry::WebView;

pub enum GUIRequest {
    ModelRate(f64),
    #[allow(dead_code)]
    ParamValue(usize, f64),
    ParamValueBatch(Vec<(usize, f64)>),
}

pub fn send_ui_event(wv: &WebView, request: GUIRequest) {
    let script = match request {
        GUIRequest::ModelRate(hz) => format!("window.__setModelRate({:.0})", hz),
        GUIRequest::ParamValue(idx, value) => format!("window.__setParam({}, {})", idx, value),
        GUIRequest::ParamValueBatch(pairs) => {
            let items: String = pairs
                .iter()
                .map(|(idx, value)| format!("{{\"id\":{},\"value\":{}}}", idx, value))
                .collect::<Vec<_>>()
                .join(",");
            format!("window.__setParamBatch([{}])", items)
        }
    };

    let _ = wv.evaluate_script(&script);
}
