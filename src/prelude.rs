
pub use pathfinder_renderer::scene::Scene;
pub use pathfinder_geometry::{
    vector::{Vector2F, vec2f},
    transform2d::Transform2F,
    rect::RectF,
};
pub use crate::{error::Error, Item, debug::DebugInfo, attrs::Attrs, DrawOptions, util::*, DrawContext};
pub use roxmltree::Node;
pub use svgtypes::{Length, LengthUnit};
pub use std::str::FromStr;

use std::collections::HashMap;
use std::sync::Arc;
pub type ItemCollection = HashMap<String, Arc<Item>>;
