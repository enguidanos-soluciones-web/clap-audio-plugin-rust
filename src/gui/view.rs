use anyrender_vello::VelloScenePainter;
use blitz_dom::DocumentConfig;
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::shell::Viewport;
use vello::Scene;

use crate::{
    actions::{Action, load_model::LoadModel as LoadModelAction},
    gui::{HitTarget, widget::Widget},
    parameters::{Parameter, Range, any::PARAMS_COUNT, blend::Blend, input_gain::InputGain, output_gain::OutputGain, tone::Tone},
    state::GUIShared,
};

const PARAM_WIDGETS: &[(&str, usize)] = &[
    ("input-gain", Parameter::<InputGain, Range>::ID),
    ("output-gain", Parameter::<OutputGain, Range>::ID),
    ("tone", Parameter::<Tone, Range>::ID),
    ("blend", Parameter::<Blend, Range>::ID),
];

const ACTION_WIDGETS: &[(&str, usize)] = &[("load-model", LoadModelAction::ID)];

pub struct View {
    pub doc: HtmlDocument,
    pub pointer: (f64, f64),
    pub element_at_pointer: Option<HitTarget>,
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
        }
    }

    pub fn set_dimensions(&mut self, width: f64, height: f64) {
        self.doc.set_viewport(Viewport {
            window_size: (width as u32, height as u32),
            ..Viewport::default()
        });
    }

    /// Called on every CursorMoved event. Single source of truth for pointer position
    /// and hit testing — sets both `self.pointer` and `self.element_at_pointer`.
    pub fn hit_test(&mut self, x: f64, y: f64) {
        self.pointer = (x, y);
        self.element_at_pointer = None;

        let Some(hit) = self.doc.hit(x as f32, y as f32) else {
            return;
        };

        let mut node_id = Some(hit.node_id);
        while let Some(id) = node_id {
            for &(dom_id, param_id) in PARAM_WIDGETS {
                if self.doc.get_element_by_id(dom_id) == Some(id) {
                    self.element_at_pointer = Some(HitTarget::Param(param_id));
                    return;
                }
            }
            for &(dom_id, action_id) in ACTION_WIDGETS {
                if self.doc.get_element_by_id(dom_id) == Some(id) {
                    self.element_at_pointer = Some(HitTarget::Action(action_id));
                    return;
                }
            }
            node_id = self.doc.get_node(id).and_then(|n| n.parent);
        }
    }

    pub fn element_at_pointer(&self) -> Option<HitTarget> {
        self.element_at_pointer
    }

    pub fn render(&mut self, scene: &mut Scene, state: &GUIShared, parameters_values: &[f64; PARAMS_COUNT]) {
        self.update_dom(state, parameters_values);

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

        self.draw_widgets(scene, parameters_values);
    }

    pub fn draw_widget(&mut self, scene: &mut Scene, widget: &dyn Widget, value: f64) {
        let Some(node_id) = self.doc.get_element_by_id(widget.dom_id()) else {
            return;
        };

        let Some(rect) = self.doc.get_client_bounding_rect(node_id) else {
            return;
        };

        widget.draw(scene, (rect.x, rect.y), (rect.width, rect.height), self.pointer, value);
    }

    pub fn draw_widgets(&mut self, scene: &mut Scene, parameters_values: &[f64; PARAMS_COUNT]) {
        self.draw_widget(
            scene,
            &Parameter::<InputGain, Range>::new(),
            parameters_values[Parameter::<InputGain, Range>::ID],
        );

        self.draw_widget(
            scene,
            &Parameter::<OutputGain, Range>::new(),
            parameters_values[Parameter::<OutputGain, Range>::ID],
        );

        self.draw_widget(
            scene,
            &Parameter::<Tone, Range>::new(),
            parameters_values[Parameter::<Tone, Range>::ID],
        );

        self.draw_widget(
            scene,
            &Parameter::<Blend, Range>::new(),
            parameters_values[Parameter::<Blend, Range>::ID],
        );
    }

    pub fn update_dom(&mut self, state: &GUIShared, parameters_values: &[f64; PARAMS_COUNT]) {
        if let Some(nam_model_rate) = state.nam_model_rate {
            if let Some(span) = self.doc.get_element_by_id("nam-model-rate") {
                let mut mutator = self.doc.mutate();
                mutator.remove_and_drop_all_children(span);
                let text = mutator.create_text_node(&format!("Model rate: {nam_model_rate:.0} Hz"));
                mutator.append_children(span, &[text]);
            }
        }

        if let Some(span) = self.doc.get_element_by_id("model-name") {
            let mut mutator = self.doc.mutate();
            mutator.remove_and_drop_all_children(span);
            if let Some(name) = &state.model_name {
                let text = mutator.create_text_node(name);
                mutator.append_children(span, &[text]);
            }
        }

        let input_id = Parameter::<InputGain, Range>::ID;
        if let Some(span) = self.doc.get_element_by_id("input-gain-db") {
            let mut mutator = self.doc.mutate();
            mutator.remove_and_drop_all_children(span);
            let text = mutator.create_text_node(&format!("{:.1} db", parameters_values[input_id]));
            mutator.append_children(span, &[text]);
        }

        let output_id = Parameter::<OutputGain, Range>::ID;
        if let Some(span) = self.doc.get_element_by_id("output-gain-db") {
            let mut mutator = self.doc.mutate();
            mutator.remove_and_drop_all_children(span);
            let text = mutator.create_text_node(&format!("{:.1} db", parameters_values[output_id]));
            mutator.append_children(span, &[text]);
        }

        let tone_id = Parameter::<Tone, Range>::ID;
        if let Some(span) = self.doc.get_element_by_id("tone-val") {
            let mut mutator = self.doc.mutate();
            mutator.remove_and_drop_all_children(span);
            let text = mutator.create_text_node(&format!("{:.1}", parameters_values[tone_id] * 5.));
            mutator.append_children(span, &[text]);
        }

        let blend_id = Parameter::<Blend, Range>::ID;
        if let Some(span) = self.doc.get_element_by_id("blend-val") {
            let mut mutator = self.doc.mutate();
            mutator.remove_and_drop_all_children(span);
            let text = mutator.create_text_node(&format!("{:.0}%", parameters_values[blend_id] * 100.));
            mutator.append_children(span, &[text]);
        }

        if let Some(node) = self.doc.get_element_by_id(LoadModelAction.dom_id()) {
            let hovered = self.element_at_pointer == Some(HitTarget::Action(LoadModelAction::ID));
            let mut mutator = self.doc.mutate();
            if hovered {
                mutator.set_style_property(node, "background-color", "#f59e0b");
                mutator.set_style_property(node, "color", "#171717");
            } else {
                mutator.remove_style_property(node, "background-color");
                mutator.remove_style_property(node, "color");
            }
        }
    }
}
