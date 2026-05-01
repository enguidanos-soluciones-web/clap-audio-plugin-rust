use crate::{
    gui::{
        colors,
        text::{TextRenderer, TextSize},
        widget::Widget,
    },
    parameters::{Action, PARAMETER_GESTURE_SINGLE_CLICK, Parameter},
    state::GuiRequest,
};
use vello::{
    Scene,
    kurbo::{Affine, RoundedRect, Stroke},
    peniko::Fill,
};

#[derive(Clone, Copy)]
pub struct LoadModel;

impl Parameter<LoadModel, Action> {
    pub const ID: usize = 4;

    pub fn new() -> Self {
        Self {
            id: Self::ID,
            name: "Load Model",
            gestures: PARAMETER_GESTURE_SINGLE_CLICK,
            behave: Action,
            _marker_type: std::marker::PhantomData,
            _marker_behaviour: std::marker::PhantomData,
        }
    }

    pub fn on_single_click(&self) -> GuiRequest {
        GuiRequest::OpenFileBrowser
    }
}

impl Widget for Parameter<LoadModel, Action> {
    fn dom_id(&self) -> &'static str {
        "load-model"
    }

    fn param_id(&self) -> usize {
        Self::ID
    }

    fn draw(
        &self,
        scene: &mut Scene,
        text: &mut TextRenderer,
        coordinates: (f64, f64),
        dimensions: (f64, f64),
        cursor: (f64, f64),
        _value: f64,
    ) {
        let rect = RoundedRect::new(
            coordinates.0 + 1.0,
            coordinates.1 + 1.0,
            coordinates.0 + dimensions.0 - 1.0,
            coordinates.1 + dimensions.1 - 1.0,
            2.0,
        );

        let hovered = cursor.0 >= coordinates.0
            && cursor.0 <= coordinates.0 + dimensions.0
            && cursor.1 >= coordinates.1
            && cursor.1 <= coordinates.1 + dimensions.1;

        let bg = if hovered { colors::neutral_700 } else { colors::neutral_800 };

        scene.fill(Fill::NonZero, Affine::IDENTITY, bg, None, &rect);
        scene.stroke(&Stroke::new(1.0), Affine::IDENTITY, colors::amber_500, None, &rect);

        text.draw_centered(
            scene,
            "LOAD MODEL",
            TextSize::Xs,
            colors::amber_500.convert(),
            coordinates,
            dimensions,
        );
    }
}
