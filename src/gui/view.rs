use taffy::prelude::*;
use vello::Scene;

use crate::gui::{
    components::{InputGainKnob, OutputGainKnob},
    layout,
    parameters::any::PARAMS_COUNT,
    widget::Widget,
};

pub struct Gui {
    width: f32,
    height: f32,
    pointer: (f32, f32),
    element_at_pointer: Option<usize>,
}

impl Gui {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            pointer: (0.0, 0.0),
            element_at_pointer: None,
        }
    }

    pub fn set_dimensions(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_pointer(&mut self, x: f32, y: f32, _is_down: bool) {
        self.pointer = (x, y);
    }

    pub fn render(&mut self, scene: &mut Scene, values: &[f32; PARAMS_COUNT]) {
        let width = self.width;
        let height = self.height;
        let pointer = self.pointer;

        self.element_at_pointer = layout::render(scene, values, width, height, pointer, |tree: &mut taffy::TaffyTree<()>, ctx| {
            let knob1 = InputGainKnob.build(tree, ctx)?;
            let knob2 = OutputGainKnob.build(tree, ctx)?;

            tree.new_with_children(
                Style {
                    display: Display::Flex,
                    align_items: Some(AlignItems::Center),
                    justify_content: Some(JustifyContent::Center),
                    gap: Size {
                        width: length(24.0),
                        height: zero(),
                    },
                    size: Size {
                        width: length(width),
                        height: length(height),
                    },
                    ..Default::default()
                },
                &[knob1, knob2],
            )
        });
    }

    pub fn element_at_pointer(&self) -> Option<usize> {
        self.element_at_pointer
    }
}
