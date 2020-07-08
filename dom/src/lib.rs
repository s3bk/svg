#![feature(trace_macros)]

#[macro_use] extern crate log;
use std::sync::Arc;
use roxmltree::NodeType;

pub mod prelude {
    pub use pathfinder_geometry::{
        vector::{Vector2F, vec2f},
        transform2d::Transform2F,
        rect::RectF,
    };
    pub use crate::{
        Item, Tag, ParseNode, TagDefs,
        animate::*,
        attrs::*,
        ellipse::*,
        error::*,
        filter::*,
        g::*,
        gradient::*,
        paint::*,
        path::*,
        polygon::*,
        rect::*,
        svg::*,
        text::*,
        util::*,
        value::*,
    };
    pub use roxmltree::Node;
    pub use svgtypes::{Length, LengthUnit};
    pub use std::str::FromStr;
    pub use crate::util::Parse;

    use std::collections::HashMap;
    use std::sync::Arc;
    pub type ItemCollection = HashMap<String, Arc<Item>>;
}

#[macro_use] mod macros;
mod animate;
mod attrs;
mod ellipse;
mod error;
mod filter;
mod g;
mod gradient;
mod paint;
mod parser;
mod path;
mod polygon;
mod rect;
mod svg;
mod text;
mod util;
mod value;

pub use prelude::*;

// enum_dispatch breaks RLS, so we do it manually
macro_rules! items {
    ($(#[$meta:meta])* pub enum $name:ident { $($($e:pat )|* => $variant:ident($data:ty), )* } { $($other:ident($other_data:ty),)* }) => {
        $( #[$meta] )*
        pub enum $name {
            $( $variant($data), )*
            $( $other($other_data), )*
        }
        impl Tag for $name {
            fn id(&self) -> Option<&str> {
                match *self {
                    $( $name::$variant ( ref tag ) => tag.id(), )*
                    _ => None,
                }
            }
            fn children(&self) -> &[Arc<Item>] {
                match *self {
                    $( $name::$variant ( ref tag ) => tag.children(), )*
                    _ => &[]
                }
            }
        }
        fn parse_element(node: &Node) -> Result<Option<Item>, Error> {
            //println!("<{:?}:{} id={:?}, ...>", node.tag_name().namespace(), node.tag_name().name(), node.attribute("id"));
            let item = match node.tag_name().name() {
                $( $($e )|* => Item::$variant(<$data>::parse_node(node)?), )*
                tag => {
                    println!("unimplemented: {}", tag);
                    return Ok(None);
                }
            };
            Ok(Some(item))
        }
    };
}

items!(
    #[derive(Debug)]
    pub enum Item {
        "path" => Path(TagPath),
        "g" => G(TagG),
        "defs" => Defs(TagDefs),
        "rect" => Rect(TagRect),
        "polygon" => Polygon(TagPolygon),
        "circle" | "ellipse" => Ellipse(TagEllipse),
        "linearGradient" => LinearGradient(TagLinearGradient),
        "radialGradient" => RadialGradient(TagRadialGradient),
        "clipPath" => ClipPath(TagClipPath),
        "filter" => Filter(TagFilter),
        "svg" => Svg(TagSvg),
        "use" => Use(TagUse),
        "symbol" => Symbol(TagSymbol),
        "text" => Text(TagText),
        "tspan" => TSpan(TagTSpan),
        "tref" => TRef(TagTRef),
    }
    {
        String(String),
    }
);

pub trait ParseNode: Sized {
    fn parse_node(node: &Node) -> Result<Self, Error>;
}

pub trait Tag: std::fmt::Debug {
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
impl ParseNode for TagDefs {
    fn parse_node(node: &Node) -> Result<TagDefs, Error> {
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

fn parse_node(node: &Node, first: bool, last: bool) -> Result<Option<Item>, Error> {
    match node.node_type() {
        NodeType::Element => parse_element(node),
        NodeType::Text => parse_text(node, first, last),
        _ => Ok(None)
    }
}

fn parse_text(node: &Node, first: bool, last: bool) -> Result<Option<Item>, Error> {
    Ok(node.text().and_then(|s| {
        let mut last_is_space = first;
        let mut processed: String = s.chars()
        .filter_map(|c| {
            if last_is_space {
                match c {
                    '\n' | '\t' | ' ' => None,
                    _ => {
                        last_is_space = false;
                        Some(c)
                    }
                }
            } else {
                match c {
                    '\n' => None,
                    '\t' | ' ' => {
                        last_is_space = true;
                        Some(' ')
                    }
                    c => Some(c)
                }
            }
        }).collect();
        if last && last_is_space && processed.len() > 0 {
            processed.pop();
        }
        if processed.len() > 0 {
            Some(Item::String(processed))
        } else {
            None
        }
    }))
}

fn parse_node_list<'a, 'i: 'a>(nodes: impl Iterator<Item=Node<'a, 'i>>) -> Result<Vec<Arc<Item>>, Error> {
    let mut items = Vec::new();
    for (first, last, node) in first_or_last_node(nodes) {
        match node.node_type() {
            NodeType::Element => {
                if let Some(item) = parse_node(&node, first, last)? {
                    items.push(Arc::new(item));
                }
            }
            _ => {}
        }
    }
    Ok(items)
}

// (first, last, node)
fn first_or_last_node<'a, 'i: 'a>(nodes: impl Iterator<Item=Node<'a, 'i>>) -> impl Iterator<Item=(bool, bool, Node<'a, 'i>)> {
    let mut nodes = nodes.enumerate().peekable();
    std::iter::from_fn(move || nodes.next().map(|(i, node)| (i == 0, nodes.peek().is_none(), node)))
}
