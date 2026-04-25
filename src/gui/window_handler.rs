use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use arc_swap::ArcSwap;
use baseview::{Event, EventStatus, MouseButton, MouseEvent, Window, WindowEvent, WindowHandler};
use vello::{Scene, kurbo::Affine};

use crate::{
    gestures::{click::ActiveClick, drag::ActiveDrag},
    gui::{gpu::Gpu, parameters::any::PARAMS_COUNT, view::Gui},
    plugin::PluginParameters,
};

pub struct GuiWindowHandler {
    gpu: Option<Gpu>,
    gui: Gui,

    parameters_rx: Arc<ArcSwap<PluginParameters>>,
    parameters_wx: Arc<Mutex<PluginParameters>>,

    cursor_drag: Option<ActiveDrag>,
    cursor_pos: baseview::Point,
    cursor_last_click: Option<(Instant, usize)>,

    width: u32,
    height: u32,
    scale: f64,
}

impl GuiWindowHandler {
    pub fn new(
        width: u32,
        height: u32,
        parameters_rx: Arc<ArcSwap<PluginParameters>>,
        parameters_wx: Arc<Mutex<PluginParameters>>,
    ) -> Self {
        Self {
            gpu: None,
            gui: Gui::new(width as f32, height as f32),
            parameters_rx,
            parameters_wx,
            cursor_pos: baseview::Point::new(0.0, 0.0),
            cursor_last_click: None,
            cursor_drag: None,
            width,
            height,
            scale: 1.0,
        }
    }
}

impl WindowHandler for GuiWindowHandler {
    fn on_frame(&mut self, _window: &mut Window) {
        let gpu = match self.gpu.as_mut() {
            Some(g) => g,
            None => return,
        };

        let params = self.parameters_rx.load();

        let values: [f32; PARAMS_COUNT] = std::array::from_fn(|i| {
            if params.main_thread_parameters_changed[i] {
                params.main_thread_parameters[i]
            } else {
                params.audio_thread_parameters[i]
            }
        });

        self.gui
            .set_pointer(self.cursor_pos.x as f32, self.cursor_pos.y as f32, self.cursor_drag.is_some());

        let mut gui_scene = Scene::new();
        self.gui.render(&mut gui_scene, &values);

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

                let Some(index) = self.gui.element_at_pointer() else {
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
                            if let Ok(mut params) = self.parameters_wx.try_lock() {
                                params.main_thread_parameters[change.index] = change.value;
                                params.main_thread_parameters_changed[change.index] = true;
                                self.parameters_rx.store(Arc::new(*params));
                            }
                        }
                        return EventStatus::Captured;
                    }
                }

                self.cursor_last_click = Some((now, index));

                let params = self.parameters_rx.load();
                let raw_value = if params.main_thread_parameters_changed[index] {
                    params.main_thread_parameters[index]
                } else {
                    params.audio_thread_parameters[index]
                };

                self.cursor_drag = ActiveDrag::from_index(index, x, y, raw_value);

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
                        if let Ok(mut params) = self.parameters_wx.try_lock() {
                            params.main_thread_parameters[change.index] = change.value;
                            params.main_thread_parameters_changed[change.index] = true;
                            self.parameters_rx.store(Arc::new(*params));
                        }
                    }
                }

                EventStatus::Captured
            }
            Event::Window(WindowEvent::Resized(info)) => {
                self.width = info.physical_size().width;
                self.height = info.physical_size().height;
                self.scale = info.scale();
                self.gui
                    .set_dimensions(self.width as f32 / self.scale as f32, self.height as f32 / self.scale as f32);
                self.gpu = Gpu::new(window, self.width, self.height);
                EventStatus::Ignored
            }
            _ => EventStatus::Ignored,
        }
    }
}
