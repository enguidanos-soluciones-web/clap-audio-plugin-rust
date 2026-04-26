use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use arc_swap::ArcSwap;
use baseview::{Event, EventStatus, MouseButton, MouseEvent, Window, WindowEvent, WindowHandler as BaseWindowHandlers};
use vello::{Scene, kurbo::Affine};

use crate::{
    gestures::{click::ActiveClick, drag::ActiveDrag},
    gui::{gpu::Gpu, parameters::any::PARAMS_COUNT, view::View},
    state::PluginParameters,
};

pub struct WindowHandler {
    gpu: Option<Gpu>,
    gui: View,

    parameters_rx: Arc<ArcSwap<PluginParameters>>,
    parameters_wx: Arc<Mutex<PluginParameters>>,

    cursor_drag: Option<ActiveDrag>,
    cursor_pos: baseview::Point,
    cursor_last_click: Option<(Instant, usize)>,

    width: u32,
    height: u32,
    scale: f64,
}

impl WindowHandler {
    pub fn new(
        width: u32,
        height: u32,
        parameters_rx: Arc<ArcSwap<PluginParameters>>,
        parameters_wx: Arc<Mutex<PluginParameters>>,
    ) -> Self {
        Self {
            gpu: None,
            gui: View::new(width as f32, height as f32),
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

impl BaseWindowHandlers for WindowHandler {
    fn on_frame(&mut self, _window: &mut Window) {
        let gpu = match self.gpu.as_mut() {
            Some(g) => g,
            None => return,
        };

        let params = self.parameters_rx.load();

        // Resolve the display value for each parameter.
        //
        // Two authoritative sources can diverge at any moment:
        //
        //   main_thread_parameters   — written by the UI (user dragging a knob).
        //                              Flagged with `main_thread_parameters_changed[i] = true`
        //                              until sync_main_to_audio() propagates it to the audio
        //                              thread on the next process() cycle.
        //
        //   audio_thread_parameters  — written by the audio thread, either when it flushes
        //                              a pending UI change, or directly via host automation /
        //                              MIDI with no UI involvement.
        //
        // Rule: prefer main_thread when the user has a pending change (changed[i] == true)
        // so the knob tracks the drag in real time. Otherwise defer to audio_thread, which
        // is the source of truth for host automation and parameter recall.
        //
        // WARNING: do not simplify this to always read one side.
        //   - Always audio  → knob freezes while dragging (lags one process() cycle behind).
        //   - Always main   → host automation and preset recall are never shown in the UI.
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

                // Same source-of-truth rule as in on_frame: if the UI has a pending
                // change for this parameter, use it as the drag start value so the
                // drag begins from where the knob visually is, not from a stale
                // audio-thread value that hasn't been acknowledged yet.
                let value = if params.main_thread_parameters_changed[index] {
                    params.main_thread_parameters[index]
                } else {
                    params.audio_thread_parameters[index]
                };

                self.cursor_drag = ActiveDrag::from_index(index, x, y, value);

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
