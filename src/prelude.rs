
pub use pathfinder_renderer::scene::Scene;
pub use pathfinder_geometry::{
    vector::{Vector2F, vec2f},
    transform2d::Transform2F,
    rect::RectF,
};
pub use crate::{
    error::Error,
    Item, Tag,
    debug::DebugInfo, 
    attrs::Attrs,
    draw::{DrawOptions, DrawContext},
    util::*,
    paint::{Color, Paint},
    animate::*,
};
pub use roxmltree::Node;
pub use svgtypes::{Length, LengthUnit};
pub use std::str::FromStr;
pub use crate::util::Parse;

use std::collections::HashMap;
use std::sync::Arc;
pub type ItemCollection = HashMap<String, Arc<Item>>;
