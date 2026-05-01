use crate::{
    channel::Sender,
    clap::clap_host_t,
    gestures::{click::ActiveClick, drag::ActiveDrag},
    gui::{HitTarget, app::dispatcher::Dispatcher, gpu::Gpu, platform::set_window_background_color, view::View},
    host_notifier::HostNotifier,
    state::{GUIShared, GuiRequest, ParamChange, ParamSnapshot},
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

    host_notifier: Arc<HostNotifier>,
    gui_shared: Arc<ArcSwap<GUIShared>>,
    gui_changes: Sender<ParamChange>,
    params_snapshot: Arc<ArcSwap<ParamSnapshot>>,

    content_scene: Scene,
    display_scene: Scene,

    cursor_drag: Option<ActiveDrag>,
    cursor_pos: baseview::Point,
    cursor_last_click: Option<(Instant, usize)>,
}

impl WindowHandler {
    pub fn new(
        window: &mut Window,
        width: u32,
        height: u32,
        host: *const clap_host_t,
        gui_shared: Arc<ArcSwap<GUIShared>>,
        gui_changes: Sender<ParamChange>,
        gui_requests: Sender<GuiRequest>,
        params_snapshot: Arc<ArcSwap<ParamSnapshot>>,
    ) -> Self {
        set_window_background_color(window);

        let host_notifier = HostNotifier::new(host);

        let dispatch: Dispatcher = {
            let gui_requests = gui_requests.clone();
            let host_notifier = Arc::clone(&host_notifier);

            Arc::new(move |req: GuiRequest| {
                let _ = gui_requests.push(req);

                host_notifier.notify();
            })
        };

        Self {
            gpu: Gpu::new(window, width, height),
            view: View::new(width as f64, height as f64, dispatch),
            width,
            height,
            scale: 1.0,

            host_notifier,
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
        self.host_notifier.notify();
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

                self.view.send_pointer_down(x, y);

                match self.view.element_at_pointer() {
                    None => {}

                    Some(HitTarget::Param(index)) => {
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
                            }
                            return EventStatus::Captured;
                        }

                        self.cursor_last_click = Some((now, index));

                        let snapshot = self.params_snapshot.load();
                        let raw = snapshot.values.get(index).copied().unwrap_or(0.0);
                        self.cursor_drag = ActiveDrag::from_index(index, x, y, raw);
                    }
                }

                EventStatus::Captured
            }
            Event::Mouse(MouseEvent::ButtonReleased {
                button: MouseButton::Left, ..
            }) => {
                self.view.send_pointer_up(self.cursor_pos.x, self.cursor_pos.y);
                self.cursor_drag = None;
                EventStatus::Captured
            }
            Event::Mouse(MouseEvent::CursorMoved { position, .. }) => {
                self.cursor_pos = position;
                self.view.hit_test(position.x, position.y);

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
