use crate::{
    channel::Sender,
    clap::clap_host_t,
    gestures::{click::ActiveClick, drag::ActiveDrag},
    gui::{gpu::Gpu, platform::set_window_background_color, view::View},
    state::{GUIShared, ParamChange, ParamSnapshot},
};
use arc_swap::ArcSwap;
use baseview::{Event, EventStatus, MouseButton, MouseEvent, Window, WindowEvent, WindowHandler as BaseWindowHandlers};
use std::{sync::Arc, time::Instant};
use vello::{Scene, kurbo::Affine};

pub struct WindowHandler {
    gpu: Option<Gpu>,
    width: u32,
    height: u32,
    scale: f64,

    view: View,

    host: *const clap_host_t,
    gui_shared: Arc<ArcSwap<GUIShared>>,
    gui_changes: Sender<ParamChange>,
    params_snapshot: Arc<ArcSwap<ParamSnapshot>>,

    content_scene: Scene,
    display_scene: Scene,

    cursor_drag: Option<ActiveDrag>,
    cursor_pos: baseview::Point,
    cursor_last_click: Option<(Instant, usize)>,
}

// SAFETY: host pointer is valid for plugin lifetime; request_callback is [thread-safe] per CLAP spec.
unsafe impl Send for WindowHandler {}

impl WindowHandler {
    pub fn new(
        window: &mut Window,
        width: u32,
        height: u32,
        host: *const clap_host_t,
        gui_shared: Arc<ArcSwap<GUIShared>>,
        gui_changes: Sender<ParamChange>,
        params_snapshot: Arc<ArcSwap<ParamSnapshot>>,
    ) -> Self {
        set_window_background_color(window);

        Self {
            gpu: Gpu::new(window, width, height),
            view: View::new(width as f64, height as f64),
            width,
            height,
            scale: 1.0,

            host,
            gui_shared,
            gui_changes,
            params_snapshot,

            content_scene: Scene::default(),
            display_scene: Scene::default(),

            cursor_pos: baseview::Point::new(0.0, 0.0),
            cursor_last_click: None,
            cursor_drag: None,
        }
    }

    fn request_host_callback(&self) {
        unsafe {
            if let Some(request_callback) = (*self.host).request_callback {
                request_callback(self.host);
            }
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
            .set_pointer(self.cursor_pos.x, self.cursor_pos.y, self.cursor_drag.is_some());

        self.content_scene.reset();
        self.view.render(&mut self.content_scene, &gui_shared, &snapshot.values);

        self.display_scene.reset();
        self.display_scene.append(&self.content_scene, Some(Affine::scale(self.scale)));
        gpu.render(&self.display_scene, self.width, self.height);
    }

    fn on_event(&mut self, window: &mut Window, event: Event) -> EventStatus {
        match event {
            Event::Mouse(MouseEvent::ButtonPressed {
                button: MouseButton::Left, ..
            }) => {
                let x = self.cursor_pos.x;
                let y = self.cursor_pos.y;

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
                            let _ = self.gui_changes.push(ParamChange {
                                id: change.index,
                                value: change.value,
                            });
                            self.request_host_callback();
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
                    if let Some(change) = cursor_drag.on_drag(position.x, position.y) {
                        let _ = self.gui_changes.push(ParamChange {
                            id: change.index,
                            value: change.value,
                        });
                        self.request_host_callback();
                    }
                }

                EventStatus::Captured
            }
            Event::Window(WindowEvent::Resized(info)) => {
                self.width = info.physical_size().width;
                self.height = info.physical_size().height;
                self.scale = info.scale();
                self.view
                    .set_dimensions(self.width as f64 / self.scale, self.height as f64 / self.scale);
                self.gpu = Gpu::new(window, self.width, self.height);

                EventStatus::Ignored
            }
            _ => EventStatus::Ignored,
        }
    }
}
