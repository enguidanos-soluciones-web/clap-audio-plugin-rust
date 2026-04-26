use anyrender_vello::VelloScenePainter;
use blitz_dom::DocumentConfig;
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::shell::Viewport;
use vello::Scene;

use crate::gui::{composition, parameters::any::PARAMS_COUNT, widget::Widget};

pub struct GUIView {
    doc: HtmlDocument,
    pointer: (f32, f32),
    element_at_pointer: Option<usize>,
}

impl GUIView {
    pub fn new(width: f32, height: f32, model_sample_rate: f64) -> Self {
        let rate_str = if model_sample_rate > 0.0 {
            format!("{:.0}", model_sample_rate)
        } else {
            "—".to_string()
        };
        let html = include_str!("layout.html").replace("{{MODEL_RATE}}", &rate_str);
        let mut doc = HtmlDocument::from_html(&html, DocumentConfig::default());

        doc.set_viewport(Viewport {
            window_size: (width as u32, height as u32),
            ..Viewport::default()
        });

        doc.resolve(0.0);

        Self {
            doc,
            pointer: (0.0, 0.0),
            element_at_pointer: None,
        }
    }

    pub fn set_dimensions(&mut self, width: f32, height: f32) {
        self.doc.set_viewport(Viewport {
            window_size: (width as u32, height as u32),
            ..Viewport::default()
        });
    }

    pub fn set_pointer(&mut self, x: f32, y: f32, _is_down: bool) {
        self.pointer = (x, y);
    }

    pub fn render(&mut self, scene: &mut Scene, values: &[f32; PARAMS_COUNT]) {
        self.doc.resolve(0.0);

        let viewport = self.doc.viewport();
        {
            let mut painter = VelloScenePainter::new(scene);

            paint_scene(
                &mut painter,
                &*self.doc,
                viewport.scale_f64(),
                viewport.window_size.0,
                viewport.window_size.1,
                0,
                0,
            );
        }

        self.element_at_pointer = None;

        composition::compose(self, scene, values);
    }

    pub fn element_at_pointer(&self) -> Option<usize> {
        self.element_at_pointer
    }

    pub fn draw_widget(&mut self, scene: &mut Scene, widget: &dyn Widget, value: f32) {
        let Some(node_id) = self.doc.get_element_by_id(widget.element_id()) else {
            return;
        };

        let Some(rect) = self.doc.get_client_bounding_rect(node_id) else {
            return;
        };

        let (px, py) = self.pointer;
        if (px as f64) >= rect.x && (px as f64) <= rect.x + rect.width && (py as f64) >= rect.y && (py as f64) <= rect.y + rect.height {
            self.element_at_pointer = Some(widget.param_id());
        }

        widget.draw(scene, rect.x, rect.y, rect.width, rect.height, widget.normalize(value));
    }
}
