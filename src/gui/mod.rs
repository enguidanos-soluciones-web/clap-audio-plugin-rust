#[allow(dead_code)]
pub mod colors;
pub mod gpu;
pub mod helpers;
pub mod platform;
pub mod text;
pub mod view;
pub mod widget;
pub mod window_handler;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HitTarget {
    Param(usize),
    Action(usize),
}
