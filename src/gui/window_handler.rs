use crate::{
    gestures::{click::ActiveClick, drag::ActiveDrag},
    gui::{gpu::Gpu, platform::set_window_background_color, view::View},
    state::{GUIShared, ParamChange, ParamSnapshot},
};
use arc_swap::ArcSwap;
use baseview::{Event, EventStatus, MouseButton, MouseEvent, Window, WindowEvent, WindowHandler as BaseWindowHandlers};
use rtrb::Producer;
use std::{sync::Arc, time::Instant};
use vello::{Scene, kurbo::Affine};

pub struct WindowHandler {
    gpu: Option<Gpu>,
    width: u32,
    height: u32,
    scale: f64,

    view: View,

    gui_shared: Arc<ArcSwap<GUIShared>>,
    gui_changes_tx: Producer<ParamChange>,
    params_snapshot: Arc<ArcSwap<ParamSnapshot>>,

    cursor_drag: Option<ActiveDrag>,
    cursor_pos: baseview::Point,
    cursor_last_click: Option<(Instant, usize)>,
}

impl WindowHandler {
    pub fn new(
        window: &mut Window,
        width: u32,
        height: u32,
        gui_shared: Arc<ArcSwap<GUIShared>>,
        gui_changes_tx: Producer<ParamChange>,
        params_snapshot: Arc<ArcSwap<ParamSnapshot>>,
    ) -> Self {
        set_window_background_color(window);

        Self {
            gpu: Gpu::new(window, width, height),
            view: View::new(width as f32, height as f32),
            width,
            height,
            scale: 1.0,

            gui_shared,
            gui_changes_tx,
            params_snapshot,

            cursor_pos: baseview::Point::new(0.0, 0.0),
            cursor_last_click: None,
            cursor_drag: None,
        }
    }
}

impl BaseWindowHandlers for WindowHandler {
    fn on_frame(&mut self, _window: &mut Window) {
        let gpu = match self.gpu.as_mut() {
            Some(g) => g,
            None => return,
        };

        let snapshot = self.params_snapshot.load();
        let gui_shared = self.gui_shared.load();

        self.view
            .set_pointer(self.cursor_pos.x as f32, self.cursor_pos.y as f32, self.cursor_drag.is_some());

        let mut gui_scene = Scene::new();
        self.view.render(&mut gui_scene, &gui_shared, &snapshot.values);

        let mut scene = Scene::new();
        scene.append(&gui_scene, Some(Affine::scale(self.scale)));
        gpu.render(&scene, self.width, self.height);
    }

    fn on_event(&mut self, window: &mut Window, event: Event) -> EventStatus {
        match event {
            Event::Mouse(MouseEvent::ButtonPressed {
                button: MouseButton::Left, ..
            }) => {
                let x = self.cursor_pos.x as f32;
                let y = self.cursor_pos.y as f32;

                let Some(index) = self.view.element_at_pointer() else {
                    return EventStatus::Captured;
                };

                let now = Instant::now();

                let is_double_click = self
                    .cursor_last_click
                    .take()
                    .is_some_and(|(t, i)| i == index && now.duration_since(t).as_millis() < 400);

                if is_double_click {
                    if let Some(clickable) = ActiveClick::from_index(index) {
                        if let Some(change) = clickable.on_double_click() {
                            let _ = self.gui_changes_tx.push(ParamChange {
                                id: change.index,
                                value: change.value,
                            });
                        }

                        return EventStatus::Captured;
                    }
                }

                self.cursor_last_click = Some((now, index));
                self.cursor_drag = ActiveDrag::from_index(index, x, y, self.params_snapshot.load().values[index]);

                EventStatus::Captured
            }
            Event::Mouse(MouseEvent::ButtonReleased {
                button: MouseButton::Left, ..
            }) => {
                self.cursor_drag = None;
                EventStatus::Captured
            }
            Event::Mouse(MouseEvent::CursorMoved { position, .. }) => {
                self.cursor_pos = position;

                if let Some(cursor_drag) = &self.cursor_drag {
                    if let Some(change) = cursor_drag.on_drag(position.x as f32, position.y as f32) {
                        let _ = self.gui_changes_tx.push(ParamChange {
                            id: change.index,
                            value: change.value,
                        });
                    }
                }

                EventStatus::Captured
            }
            Event::Window(WindowEvent::Resized(info)) => {
                self.width = info.physical_size().width;
                self.height = info.physical_size().height;
                self.scale = info.scale();
                self.view
                    .set_dimensions(self.width as f32 / self.scale as f32, self.height as f32 / self.scale as f32);
                self.gpu = Gpu::new(window, self.width, self.height);

                EventStatus::Ignored
            }
            _ => EventStatus::Ignored,
        }
    }
}
