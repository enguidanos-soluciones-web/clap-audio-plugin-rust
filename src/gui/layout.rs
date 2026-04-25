use taffy::{NodeId, TaffyError, TaffyTree, prelude::*};
use vello::Scene;

use crate::gui::widget::{BoundingBox, LayoutContext};

pub fn render<F>(scene: &mut Scene, values: &[f32], width: f32, height: f32, pointer: (f32, f32), build: F) -> Option<usize>
where
    F: FnOnce(&mut TaffyTree<()>, &LayoutContext) -> Result<NodeId, TaffyError>,
{
    let mut tree: TaffyTree<()> = TaffyTree::new();
    let ctx = LayoutContext::new(values);

    let root = build(&mut tree, &ctx).ok()?;

    tree.compute_layout(
        root,
        Size {
            width: AvailableSpace::Definite(width),
            height: AvailableSpace::Definite(height),
        },
    )
    .ok()?;

    let (hittable, drawables) = ctx.into_parts();

    for (node, draw_fn) in drawables {
        let (x, y) = absolute_location(&tree, node);
        let Ok(layout) = tree.layout(node) else { continue };
        draw_fn(
            scene,
            BoundingBox {
                x,
                y,
                width: layout.size.width,
                height: layout.size.height,
            },
        );
    }

    let mut element_at_pointer = None;

    for (node, param_index) in hittable {
        let Ok(layout) = tree.layout(node) else { continue };
        let (lx, ly) = absolute_location(&tree, node);
        let lw = layout.size.width;
        let lh = layout.size.height;

        if pointer.0 >= lx && pointer.0 <= lx + lw && pointer.1 >= ly && pointer.1 <= ly + lh {
            element_at_pointer = Some(param_index);
        }
    }

    element_at_pointer
}

fn absolute_location(tree: &TaffyTree<()>, node: NodeId) -> (f32, f32) {
    let mut x = 0.0f32;
    let mut y = 0.0f32;

    let mut current = node;

    while let Ok(layout) = tree.layout(current) {
        x += layout.location.x;
        y += layout.location.y;

        match tree.parent(current) {
            Some(parent) => current = parent,
            None => break,
        }
    }

    (x, y)
}
