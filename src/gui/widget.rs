use std::cell::RefCell;

use taffy::NodeId;
use vello::Scene;

pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

type LayoutContextHittable = RefCell<Vec<(NodeId, usize)>>;
type LayoutContextDrawabled = RefCell<Vec<(NodeId, Box<dyn Fn(&mut Scene, BoundingBox)>)>>;

pub struct LayoutContext<'a> {
    pub values: &'a [f32],
    hittable: LayoutContextHittable,
    drawables: LayoutContextDrawabled,
}

impl<'a> LayoutContext<'a> {
    pub fn new(values: &'a [f32]) -> Self {
        Self {
            values,
            hittable: RefCell::new(Vec::new()),
            drawables: RefCell::new(Vec::new()),
        }
    }

    pub fn register_hittable(&self, node: NodeId, param_index: usize) {
        self.hittable.borrow_mut().push((node, param_index));
    }

    pub fn register_drawable(&self, node: NodeId, draw: Box<dyn Fn(&mut Scene, BoundingBox)>) {
        self.drawables.borrow_mut().push((node, draw));
    }

    pub fn into_parts(self) -> (Vec<(NodeId, usize)>, Vec<(NodeId, Box<dyn Fn(&mut Scene, BoundingBox)>)>) {
        (self.hittable.into_inner(), self.drawables.into_inner())
    }
}

pub trait Widget {
    fn build(&self, tree: &mut taffy::TaffyTree<()>, ctx: &LayoutContext) -> Result<NodeId, taffy::TaffyError>;
}
