#[cfg(feature="debug")]
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

use enum_dispatch::enum_dispatch;
use roxmltree::{Node, NodeType};
use pathfinder_renderer::{
    scene::{Scene}
};
use std::sync::Arc;

mod prelude;

#[macro_use] mod util;
use util::*;

mod path;
use path::*;

mod rect;
use rect::TagRect;

mod polygon;
use polygon::TagPolygon;

mod ellipse;
use ellipse::TagEllipse;

mod debug;

mod error;
mod attrs;
use attrs::*;

mod gradient;
use gradient::{TagLinearGradient, TagRadialGradient};

mod filter;
use filter::*;

mod g;
use g::*;

mod draw;
pub use draw::{DrawContext, DrawOptions};

mod svg;
use svg::TagSvg;
pub use svg::Svg;

#[cfg(feature="text")]
mod text;

#[cfg(feature="text")]
use text::*;

use prelude::*;

#[enum_dispatch]
#[derive(Debug)]
pub enum Item {
    Path(TagPath),
    G(TagG),
    Defs(TagDefs),
    Rect(TagRect),
    Polygon(TagPolygon),
    Ellipse(TagEllipse),
    LinearGradient(TagLinearGradient),
    RadialGradient(TagRadialGradient),
    ClipPath(TagClipPath),
    Filter(TagFilter),
    Svg(TagSvg),
}

#[enum_dispatch(Item)]
pub trait Tag: Sized + std::fmt::Debug {
    fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {}
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> { None }
    fn id(&self) -> Option<&str> { None }
    fn children(&self) -> &[Arc<Item>] { &[] }
}

#[derive(Debug)]
pub struct TagDefs {
    items: Vec<Arc<Item>>,
}
impl Tag for TagDefs {
    fn children(&self) -> &[Arc<Item>] {
        &self.items
    }
}
impl TagDefs {
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagDefs, Error> {
        let items = parse_node_list(node.children())?;
        Ok(TagDefs { items })
    }
}

fn link(ids: &mut ItemCollection, item: &Arc<Item>) {
    if let Some(id) = item.id() {
        ids.insert(id.into(), item.clone());
    }
    for child in item.children() {
        link(ids, child);
    }
}


fn parse_node_list<'a, 'i: 'a>(nodes: impl Iterator<Item=Node<'a, 'i>>) -> Result<Vec<Arc<Item>>, Error> {
    let mut items = Vec::new();
    for node in nodes {
        match node.node_type() {
            NodeType::Element => {
                if let Some(item) = parse_node(&node)? {
                    items.push(Arc::new(item));
                }
            }
            _ => {}
        }
    }
    Ok(items)
}

fn parse_node<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<Option<Item>, Error> {
    //println!("<{:?}:{} id={:?}, ...>", node.tag_name().namespace(), node.tag_name().name(), node.attribute("id"));
    Ok(match node.tag_name().name() {
        "title" | "desc" | "metadata" => None,
        "defs" => Some(Item::Defs(TagDefs::parse(node)?)),
        "path" => Some(Item::Path(TagPath::parse(node)?)),
        "g" => Some(Item::G(TagG::parse(node)?)),
        "rect" => Some(Item::Rect(TagRect::parse(node)?)),
        "polygon" => Some(Item::Polygon(TagPolygon::parse(node)?)),
        "ellipse" | "circle" => Some(Item::Ellipse(TagEllipse::parse(node)?)),
        "linearGradient" => Some(Item::LinearGradient(TagLinearGradient::parse(node)?)),
        "radialGradient" => Some(Item::RadialGradient(TagRadialGradient::parse(node)?)),
        "clipPath" => Some(Item::ClipPath(TagClipPath::parse(node)?)),
        "filter" => Some(Item::Filter(TagFilter::parse(node)?)),
        "svg" => Some(Item::Svg(TagSvg::parse(node)?)),
        tag => {
            println!("unimplemented: {}", tag);
            None
        }
    })
}

