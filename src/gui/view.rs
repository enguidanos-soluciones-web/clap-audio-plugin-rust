use crate::{
    gui::{
        HitTarget,
        app::{dispatcher::Dispatcher, layout::Layout, state::AppState},
        widget::Widget,
    },
    parameters::{Parameter, Range, any::PARAMS_COUNT, blend::Blend, input_gain::InputGain, output_gain::OutputGain, tone::Tone},
    state::GUIShared,
};
use anyrender_vello::VelloScenePainter;
use baseview::MouseButton;
use blitz_dom::{Document, DocumentConfig};
use blitz_paint::paint_scene;
use blitz_traits::{
    events::{BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, PointerCoords, PointerDetails, UiEvent},
    shell::Viewport,
};
use dioxus::prelude::{Signal, WritableExt};
use dioxus_core::{ScopeId, VirtualDom};
use dioxus_native_dom::DioxusDocument;
use keyboard_types::Modifiers;
use vello::Scene;

const PARAM_WIDGETS: &[(&str, usize)] = &[
    ("input-gain", Parameter::<InputGain, Range>::ID),
    ("output-gain", Parameter::<OutputGain, Range>::ID),
    ("tone", Parameter::<Tone, Range>::ID),
    ("blend", Parameter::<Blend, Range>::ID),
];

pub struct View {
    pub doc: DioxusDocument,
    pub app_state: Signal<AppState>,
    pub pointer: (f64, f64),
    pub element_at_pointer: Option<HitTarget>,
    pub held_buttons: MouseEventButtons,
}

impl View {
    pub fn new(width: f64, height: f64, dispatch: Dispatcher) -> Self {
        let vdom = VirtualDom::new(Layout);

        // Create the signal inside the Dioxus runtime so reactive subscriptions work.
        // Signal writes send SchedulerMsg to the vdom channel — poll() picks these up.
        let app_state: Signal<AppState> = vdom.in_runtime(|| Signal::new_in_scope(AppState::default(), ScopeId::ROOT));

        vdom.provide_root_context(app_state);
        vdom.provide_root_context(dispatch);

        let css = include_str!("app/style/output.css");
        let mut doc = DioxusDocument::new(vdom, DocumentConfig::default());
        doc.create_head_element("style", &[], &Some(css.to_string()));

        {
            let mut inner = doc.inner_mut();
            inner.set_viewport(Viewport {
                window_size: (width as u32, height as u32),
                ..Viewport::default()
            });
        }

        doc.initial_build();

        {
            let mut inner = doc.inner_mut();
            inner.resolve(0.0);
        }

        Self {
            doc,
            app_state,
            pointer: (0.0, 0.0),
            element_at_pointer: None,
            held_buttons: MouseEventButtons::None,
        }
    }

    pub fn set_dimensions(&mut self, width: f64, height: f64) {
        let mut inner = self.doc.inner_mut();
        inner.set_viewport(Viewport {
            window_size: (width as u32, height as u32),
            ..Viewport::default()
        });
        // Mark all nodes dirty so flush_styles_to_layout re-propagates
        // viewport-dependent sizes (h-full, w-full, flex-1) on next resolve().
        let root_id = inner.root_element().id;
        inner.get_node(root_id).map(|n| n.set_dirty_descendants());
    }

    pub fn send_pointer_down(&mut self, x: f64, y: f64, button: MouseButton) {
        self.held_buttons |= mouse_button_mask(button);
        let ui_event = UiEvent::PointerDown(self.make_pointer_event(x, y, mouse_button(button)));
        self.doc.handle_ui_event(ui_event);
    }

    pub fn send_pointer_up(&mut self, x: f64, y: f64, button: MouseButton) {
        self.held_buttons &= !mouse_button_mask(button);
        let ui_event = UiEvent::PointerUp(self.make_pointer_event(x, y, mouse_button(button)));
        self.doc.handle_ui_event(ui_event);
    }

    fn make_pointer_event(&self, x: f64, y: f64, button: MouseEventButton) -> BlitzPointerEvent {
        let coords = PointerCoords {
            page_x: x as f32,
            page_y: y as f32,
            screen_x: x as f32,
            screen_y: y as f32,
            client_x: x as f32,
            client_y: y as f32,
        };

        BlitzPointerEvent {
            id: BlitzPointerId::Mouse,
            is_primary: true,
            coords,
            button,
            buttons: self.held_buttons,
            mods: Modifiers::empty(),
            details: PointerDetails::default(),
        }
    }

    /// Called on every CursorMoved event. Single source of truth for pointer position
    /// and hit testing — sets both `self.pointer` and `self.element_at_pointer`.
    pub fn hit_test(&mut self, x: f64, y: f64) {
        self.pointer = (x, y);
        self.element_at_pointer = None;

        let ui_event = UiEvent::PointerMove(self.make_pointer_event(x, y, MouseEventButton::Main));
        self.doc.handle_ui_event(ui_event);

        let inner = self.doc.inner();
        let Some(hit) = inner.hit(x as f32, y as f32) else {
            return;
        };

        let mut node_id = Some(hit.node_id);
        while let Some(id) = node_id {
            for &(dom_id, param_id) in PARAM_WIDGETS {
                if inner.get_element_by_id(dom_id) == Some(id) {
                    self.element_at_pointer = Some(HitTarget::Param(param_id));
                    return;
                }
            }
            node_id = inner.get_node(id).and_then(|n| n.parent);
        }
    }

    pub fn render(&mut self, scene: &mut Scene, state: &GUIShared, parameters_values: &[f64; PARAMS_COUNT]) {
        self.update_app_state(state, parameters_values);
        // Signal write (in update_app_state) already queued a SchedulerMsg — poll processes it.
        self.doc.poll(None);

        {
            let mut inner = self.doc.inner_mut();
            inner.resolve(0.0);
        }

        {
            let inner = self.doc.inner();
            let viewport = inner.viewport().clone();
            let mut painter = VelloScenePainter::new(scene);
            paint_scene(
                &mut painter,
                &*inner,
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
        let inner = self.doc.inner();
        let Some(node_id) = inner.get_element_by_id(widget.dom_id()) else {
            return;
        };

        let Some(rect) = inner.get_client_bounding_rect(node_id) else {
            return;
        };

        drop(inner);

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

    pub fn update_app_state(&mut self, state: &GUIShared, parameters_values: &[f64; PARAMS_COUNT]) {
        // Write inside the Dioxus runtime so the reactive system tracks the change
        // and sends SchedulerMsg to the vdom channel.
        let mut app_state = self.app_state;

        self.doc.vdom.in_runtime(|| {
            let mut s = app_state.write();
            s.params = *parameters_values;
            s.model_name = state.model_name.clone();
            s.model_rate = state.nam_model_rate;
        });
    }
}

fn mouse_button(button: MouseButton) -> MouseEventButton {
    match button {
        MouseButton::Left => MouseEventButton::Main,
        MouseButton::Middle => MouseEventButton::Auxiliary,
        MouseButton::Right => MouseEventButton::Secondary,
        MouseButton::Back => MouseEventButton::Fourth,
        MouseButton::Forward => MouseEventButton::Fifth,
        MouseButton::Other(_) => MouseEventButton::Main,
    }
}

fn mouse_button_mask(button: MouseButton) -> MouseEventButtons {
    match button {
        MouseButton::Left => MouseEventButtons::Primary,
        MouseButton::Right => MouseEventButtons::Secondary,
        MouseButton::Middle => MouseEventButtons::Auxiliary,
        MouseButton::Back => MouseEventButtons::Fourth,
        MouseButton::Forward => MouseEventButtons::Fifth,
        MouseButton::Other(_) => MouseEventButtons::None,
    }
}
