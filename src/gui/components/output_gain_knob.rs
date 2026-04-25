use taffy::{NodeId, TaffyError, TaffyTree, prelude::*};

use crate::gui::{
    atoms::knob,
    parameter::{Parameter, Range},
    parameters::output_gain::OutputGain,
    widget::{LayoutContext, Widget},
};

pub struct OutputGainKnob;

impl Widget for OutputGainKnob {
    fn build(&self, tree: &mut TaffyTree<()>, ctx: &LayoutContext) -> Result<NodeId, TaffyError> {
        let param = Parameter::<OutputGain, Range>::new();
        let normalized = param.normalize(ctx.values[param.id] as f64);

        let knob_node = tree.new_leaf(Style {
            size: Size {
                width: length(80.0),
                height: length(80.0),
            },
            ..Default::default()
        })?;

        let label_node = tree.new_leaf(Style {
            size: Size {
                width: auto(),
                height: length(14.0),
            },
            ..Default::default()
        })?;

        let col = tree.new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: Some(AlignItems::Center),
                gap: Size {
                    width: zero(),
                    height: length(8.0),
                },
                size: Size {
                    width: length(100.0),
                    height: length(130.0),
                },
                padding: Rect {
                    top: length(16.0),
                    bottom: length(16.0),
                    left: zero(),
                    right: zero(),
                },
                ..Default::default()
            },
            &[knob_node, label_node],
        )?;

        ctx.register_drawable(
            knob_node,
            Box::new(move |scene, bb| {
                let cx = bb.x as f64 + bb.width as f64 / 2.0;
                let cy = bb.y as f64 + bb.height as f64 / 2.0;
                let r = (bb.width.min(bb.height) as f64 / 2.0) - 4.0;
                knob::draw(scene, cx, cy, r, normalized);
            }),
        );

        ctx.register_hittable(knob_node, param.id);

        Ok(col)
    }
}
