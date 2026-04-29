use anyrender_vello::VelloScenePainter;
use blitz_dom::DocumentConfig;

use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::shell::Viewport;
use vello::Scene;

use crate::{
    gui::{composition, widget::Widget},
    parameters::any::PARAMS_COUNT,
    state::GUIShared,
};

pub struct View {
    pub doc: HtmlDocument,
    pub pointer: (f64, f64),
    pub element_at_pointer: Option<usize>,

    dom_dirty: bool,
    prev_params: [f64; PARAMS_COUNT],
    prev_gui_shared: GUIShared,
}

impl View {
    pub fn new(width: f64, height: f64) -> Self {
        let html = include_str!("layout/index.html").replace("%STYLESHEET%", include_str!("layout/output.css"));

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
            dom_dirty: false,
            // NaN guarantees first-frame DOM update via != comparison
            prev_params: [f64::NAN; PARAMS_COUNT],
            prev_gui_shared: GUIShared::default(),
        }
    }

    pub fn set_dimensions(&mut self, width: f64, height: f64) {
        self.doc.set_viewport(Viewport {
            window_size: (width as u32, height as u32),
            ..Viewport::default()
        });
        self.dom_dirty = true;
    }

    pub fn set_pointer(&mut self, x: f64, y: f64, _is_down: bool) {
        self.pointer = (x, y);
    }

    pub fn render(&mut self, scene: &mut Scene, state: &GUIShared, parameters_values: &[f64; PARAMS_COUNT]) {
        // 1. Mutate DOM text nodes only when values changed
        let prev_params = self.prev_params;
        let prev_gui_shared = self.prev_gui_shared;
        let dom_mutated = composition::update_dom(self, state, parameters_values, &prev_gui_shared, &prev_params);

        if dom_mutated {
            self.prev_params = *parameters_values;
            self.prev_gui_shared = *state;
        }

        // 2. Recompute CSS layout only when DOM is dirty (mutation or resize)
        if dom_mutated || self.dom_dirty {
            self.doc.resolve(0.0);
            self.dom_dirty = false;
        }

        // 3. Paint HTML/CSS layer
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

        // 4. Draw Vello widgets (pure vector, no DOM dependency)
        self.element_at_pointer = None;
        composition::draw_widgets(self, scene, parameters_values);
    }

    pub fn element_at_pointer(&self) -> Option<usize> {
        self.element_at_pointer
    }

    pub fn draw_widget(&mut self, scene: &mut Scene, widget: &dyn Widget, value: f64) {
        let Some(node_id) = self.doc.get_element_by_id(widget.element_id()) else {
            return;
        };

        let Some(rect) = self.doc.get_client_bounding_rect(node_id) else {
            return;
        };

        let (px, py) = (self.pointer.0 as f64, self.pointer.1 as f64);
        let within_x = px >= rect.x && px <= rect.x + rect.width;
        let within_y = py >= rect.y && py <= rect.y + rect.height;
        if within_x && within_y {
            self.element_at_pointer = Some(widget.param_id());
        }

        widget.draw(scene, rect.x, rect.y, rect.width, rect.height, widget.normalize(value));
    }
}
