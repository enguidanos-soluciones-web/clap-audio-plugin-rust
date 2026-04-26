use std::sync::{Arc, Mutex};

use anyrender_vello::VelloScenePainter;
use blitz_dom::DocumentConfig;
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::shell::Viewport;
use vello::Scene;

use crate::{
    gui::{composition, parameters::any::PARAMS_COUNT, widget::Widget},
    processors::handle_gui_event::{GUIEvent, handle_gui_event},
};

pub struct View {
    pub doc: HtmlDocument,
    pub pointer: (f32, f32),
    pub element_at_pointer: Option<usize>,
}

impl View {
    pub fn new(width: f32, height: f32) -> Self {
        let html = include_str!("layout.html");

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

    pub fn render(&mut self, scene: &mut Scene, parameters_values: &[f32; PARAMS_COUNT], queue: Arc<Mutex<Vec<GUIEvent>>>) {
        if let Ok(mut queue) = queue.try_lock() {
            for msg in queue.drain(..) {
                handle_gui_event(self, msg);
            }
        }
        drop(queue);

        self.doc.resolve(0.0);

        let viewport = self.doc.viewport();
        {
            // `VelloScenePainter` implements `PaintScene` — it translates the
            // abstract drawing commands from `paint_scene` into Vello vector ops
            // that land in `scene`. Scoped so the borrow of `scene` ends here,
            // freeing it for the widget drawing that follows.
            let mut painter = VelloScenePainter::new(scene);

            // `paint_scene` walks the resolved DOM and pushes drawing commands
            // (backgrounds, borders, text) into `painter`. This renders the HTML/CSS
            // layer: the header, labels ("Gain", "Master"), and the plugin background.
            // The knob graphics are NOT drawn here — that happens via `composition::compose`
            // below, using the rects that CSS layout already computed for the .knob divs.
            //
            // Parameters:
            //   &mut painter          — destination for drawing commands (Vello backend)
            //   &*self.doc            — the resolved Blitz DOM (deref HtmlDocument → BaseDocument)
            //   viewport.scale_f64()  — HiDPI/Retina scale factor (1.0 on standard, 2.0 on Retina)
            //   viewport.window_size  — pixel dimensions of the window, needed to clip the output
            //   x_offset / y_offset   — scroll offsets; 0 because the plugin UI does not scroll
            let x_offset = 0;
            let y_offset = 0;

            paint_scene(
                &mut painter,
                &*self.doc,
                viewport.scale_f64(),
                viewport.window_size.0,
                viewport.window_size.1,
                x_offset,
                y_offset,
            );
        }

        self.element_at_pointer = None;

        composition::compose(self, scene, parameters_values);
    }

    pub fn element_at_pointer(&self) -> Option<usize> {
        self.element_at_pointer
    }

    pub fn draw_widget(&mut self, scene: &mut Scene, widget: &dyn Widget, value: f32) {
        // Each widget declares an HTML element ID (via `element_id()`).
        // That ID must exist in `layout.html` so the Blitz DOM knows about the widget.
        // Without this node the layout engine has no anchor for the widget,
        // so there is nothing to draw — skip it.
        let Some(node_id) = self.doc.get_element_by_id(widget.element_id()) else {
            return;
        };

        // `node_id` is just an opaque handle into the DOM tree.
        // `get_client_bounding_rect` runs the layout engine result for that node
        // and returns the pixel rect it occupies in the window after CSS layout.
        // This can be `None` if the node is not yet laid out (e.g. `display:none`
        // or `resolve()` hasn't been called yet), in which case skip drawing.
        //
        // The rect dimensions come entirely from CSS (e.g. `.knob { width: 80px; height: 80px }` in
        // layout.html). The HTML elements are intentionally empty — they carry no visual content of
        // their own. CSS is the only thing giving them a size; without it the rect would be 0×0
        // and Vello would draw the widget collapsed to a point.
        let Some(rect) = self.doc.get_client_bounding_rect(node_id) else {
            return;
        };

        // Hit-test: check if the pointer is inside the widget's bounding rect.
        // Pointer coords are f32; rect coords are f64, so cast before comparing.
        let (px, py) = (self.pointer.0 as f64, self.pointer.1 as f64);
        let within_x = px >= rect.x && px <= rect.x + rect.width;
        let within_y = py >= rect.y && py <= rect.y + rect.height;
        if within_x && within_y {
            // Pointer is over this widget — record its param_id so callers
            // can query which parameter the user is hovering or interacting with.
            self.element_at_pointer = Some(widget.param_id());
        }

        widget.draw(scene, rect.x, rect.y, rect.width, rect.height, widget.normalize(value));
    }
}
