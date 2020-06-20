#[cfg(feature="debug")]
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

use roxmltree::{Document, Node, NodeType};
use pathfinder_content::{
    outline::{Outline},
    stroke::{OutlineStrokeToFill, StrokeStyle, LineCap, LineJoin},
    fill::{FillRule},
    gradient::Gradient,
};
use pathfinder_renderer::{
    scene::{Scene, DrawPath, ClipPath, ClipPathId},
    paint::Paint as PaPaint,
};
use pathfinder_color::ColorU;
use libflate::gzip::Decoder;
use svgtypes::{Length, LengthListParser, Color};
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

mod draw;
use draw::*;

#[cfg(feature="text")]
mod text;

#[cfg(feature="text")]
use text::*;

use prelude::*;

#[derive(Debug)]
pub struct Svg {
    items: Vec<Arc<Item>>,
    view_box: Option<Rect>,
    named_items: ItemCollection,
}
impl Svg {
    pub fn compose(&self) -> Scene {
        let mut scene = Scene::new();
        let ctx = DrawContext {
            svg: self,
            dpi: 75.0,

            #[cfg(feature="debug")]
            debug_font: Arc::new(FontCollection::debug()),
            #[cfg(feature="debug")]
            debug: false,
        };
        let options = DrawOptions::new(&ctx);
        if let Some(ref r) = self.view_box {
            scene.set_view_box(options.transform * r.as_rectf());
        }
        self.compose_to(&mut scene, options);
        scene
    }
    pub fn compose_to(&self, scene: &mut Scene, options: DrawOptions) {
        for item in &self.items {
            item.compose_to(scene, &options);
        }
    }
    pub fn parse<'a>(doc: &'a Document) -> Result<Svg, Error> {
        let root = doc.root_element();
        assert!(root.has_tag_name("svg"));
        let view_box = root.attribute("viewBox").map(Rect::parse).transpose()?;
        let width = root.attribute("width").map(length).transpose()?;
        let height = root.attribute("height").map(length).transpose()?;
    
        let view_box = match (view_box, width, height) {
            (Some(r), _, _) => Some(r),
            (None, Some(w), Some(h)) => Some(Rect::from_size(w, h)),
            _ => None
        };

        let items = parse_node_list(root.children())?;
    
        let mut named_items = ItemCollection::new();
        for item in &items {
            link(&mut named_items, item);
        }
    
        Ok(Svg { items, view_box, named_items })
    }
    pub fn from_str(text: &str) -> Result<Svg, Error> {
        timed!("parse xml", {
            let doc = Document::parse(text)?;
        });
        timed!("build svg", {
            Svg::parse(&doc)
        })
    }
    pub fn from_data(data: &[u8]) -> Result<Svg, Error> {
        if data.starts_with(&[0x1f, 0x8b]) {
            timed!("inflate", {
                use std::io::Read;
                let mut decoder = Decoder::new(data).map_err(Error::Gzip)?;
                let mut decoded_data = Vec::new();
                decoder.read_to_end(&mut decoded_data).map_err(Error::Gzip)?;
            });
            let text = std::str::from_utf8(&decoded_data)?;
            Self::from_str(text)
        } else {
            let text = std::str::from_utf8(data)?;
            Self::from_str(text)
        }
    }
}
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
}

impl Item {
    fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        match *self {
            Item::G(ref tag) => tag.compose_to(scene, &options),
            Item::Path(ref tag) => tag.compose_to(scene, &options),
            Item::Rect(ref tag) => tag.compose_to(scene, &options),
            Item::Polygon(ref tag) => tag.compose_to(scene, &options),
            Item::Ellipse(ref tag) => tag.compose_to(scene, &options),
            _ => {}
        }
    }
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        match *self {
            Item::G(ref tag) => tag.bounds(&options),
            Item::Path(ref tag) => tag.bounds(&options),
            Item::Rect(ref tag) => tag.bounds(&options),
            Item::Polygon(ref tag) => tag.bounds(&options),
            Item::Ellipse(ref tag) => tag.bounds(&options),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct TagG {
    items: Vec<Arc<Item>>,
    attrs: Attrs,
}
impl TagG {
    pub fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if !self.attrs.display {
            return None;
        }

        let options = options.apply(&self.attrs);
        max_bounds(self.items.iter().flat_map(|item| item.bounds(&options)))
    }
    pub fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        if !self.attrs.display {
            return;
        }

        let options = options.apply(&self.attrs);

        if let Some(ref filter_id) = self.attrs.filter {
            let bounds = match max_bounds(self.items.iter().flat_map(|item| item.bounds(&options))) {
                Some(b) => b,
                None => {
                    println!("no bounds for {:?}", self);
                    return;
                }
            };
            match options.ctx.resolve(&filter_id).map(|i| &**i) {
                Some(Item::Filter(filter)) => {
                    filter.apply(scene, &options, bounds, |scene, options| {
                        for item in &self.items {
                            item.compose_to(scene, options);
                        }
                    });
                    return;
                },
                r => println!("expected filter, got {:?}", r)
            }
        }

        for item in &self.items {
            item.compose_to(scene, &options);
        }
    }
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagG, Error> {
        let attrs = Attrs::parse(node)?;
        let items = parse_node_list(node.children())?;
        Ok(TagG { items, attrs })
    }
}
#[derive(Debug)]
pub struct TagDefs {
    items: Vec<Arc<Item>>,
}
impl TagDefs {
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagDefs, Error> {
        let items = parse_node_list(node.children())?;
        Ok(TagDefs { items })
    }
}

fn link(ids: &mut ItemCollection, item: &Arc<Item>) {
    match &**item {
        Item::G(g) => g.items.iter().for_each(|item| link(ids, item)),
        Item::Defs(defs) => defs.items.iter().for_each(|item| link(ids, item)),
        Item::LinearGradient(TagLinearGradient { id: Some(id), .. }) |
        Item::RadialGradient(TagRadialGradient { id: Some(id), .. }) |
        Item::ClipPath(TagClipPath { id: Some(id), .. }) |
        Item::Filter(TagFilter { id: Some(id), .. }) |
        Item::Path(TagPath { id: Some(id ), .. }) |
        Item::Rect(TagRect { id: Some(id), .. }) |
        Item::Polygon(TagPolygon { id: Some(id), .. }) |
        Item::Ellipse(TagEllipse { id: Some(id), .. })
        => {
             ids.insert(id.clone(), item.clone());
        }
        _ => {}
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
        tag => {
            println!("unimplemented: {}", tag);
            None
        }
    })
}

