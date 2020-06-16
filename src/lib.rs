#[cfg(feature="debug")]
#[macro_use] extern crate lazy_static;

use roxmltree::{Document, Node, NodeType};
use pathfinder_content::{
    outline::{Outline},
    stroke::{OutlineStrokeToFill, StrokeStyle, LineCap, LineJoin},
    fill::{FillRule},
    gradient::Gradient,
};
use pathfinder_renderer::{
    scene::{Scene, DrawPath},
    paint::Paint as PaPaint,
};
use pathfinder_color::ColorU;

use svgtypes::{Length, LengthListParser, Color};
use std::sync::Arc;

mod prelude;

mod util;
use util::*;

mod path;
use path::TagPath;

mod rect;
use rect::TagRect;

mod polygon;
use polygon::TagPolygon;

mod debug;

mod error;
mod attrs;
use attrs::*;

mod gradient;
use gradient::{TagLinearGradient, TagRadialGradient};

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
        if let Some(ref r) = self.view_box {
            scene.set_view_box(dbg!(r.as_rectf()));
        }
        let ctx = DrawContext {
            svg: self,
            dpi: 75.0,

            #[cfg(feature="debug")]
            debug_font: Arc::new(FontCollection::debug()),
            #[cfg(feature="debug")]
            debug: false,
        };
        self.compose_to(&mut scene, DrawOptions::new(&ctx));
        scene
    }
    pub fn compose_to(&self, scene: &mut Scene, options: DrawOptions) {
        for item in &self.items {
            item.compose_to(scene, &options);
        }
    }
    pub fn parse<'a>(doc: &'a Document) -> Result<Svg, Error<'a>> {
        let root = doc.root_element();
        dbg!(root.tag_name());
        assert!(root.has_tag_name("svg"));
        let view_box = root.attribute("viewBox").map(Rect::parse).transpose()?;
    
        let items = parse_node_list(root.children())?;
    
        let mut named_items = ItemCollection::new();
        for item in &items {
            link(&mut named_items, item);
        }
    
        Ok(Svg { items, view_box, named_items })
    }
}

pub struct DrawContext<'a> {
    svg: &'a Svg,

    #[cfg(feature="debug")]
    debug_font: Arc<FontCollection>,
    #[cfg(feature="debug")]
    debug: bool,

    dpi: f32,
}
impl<'a> DrawContext<'a> {
    pub fn resolve(&self, id: &str) -> Option<&Arc<Item>> {
        self.svg.named_items.get(id)
    }
}

#[derive(Clone)]
pub struct DrawOptions<'a> {
    ctx: &'a DrawContext<'a>,

    fill: Option<Paint>,
    fill_rule: FillRule,
    fill_opacity: f32,

    stroke: Option<Paint>,
    stroke_style: StrokeStyle,
    stroke_opacity: f32,

    transform: Transform2F,

}
impl<'a> DrawOptions<'a> {
    pub fn new(ctx: &'a DrawContext<'a>) -> DrawOptions<'a> {
        DrawOptions {
            ctx,
            fill: None,
            fill_rule: FillRule::EvenOdd,
            fill_opacity: 1.0,
            stroke: None,
            stroke_opacity: 1.0,
            stroke_style: StrokeStyle {
                line_width: 1.0,
                line_cap: LineCap::Butt,
                line_join: LineJoin::Bevel,
            },
            transform: Transform2F::default(),
        }
    }
    pub fn transform(mut self, transform: Transform2F) -> DrawOptions<'a> {
        self.transform = transform;
        self
    }
    #[cfg(feature="debug")]
    pub fn debug(mut self) -> DrawOptions {
        self.debug = true;
        self
    }
    fn resolve_paint(&self, paint: &Option<Paint>, opacity: f32) -> Option<PaPaint> {
        match *paint {
            Some(Paint::Color(Color { red, green, blue })) => Some(PaPaint::from_color(ColorU::new(red, green, blue, (255. * opacity) as u8))),
            Some(Paint::Ref(ref id)) => match self.ctx.svg.named_items.get(id).map(|arc| &**arc) {
                Some(Item::LinearGradient(ref gradient)) => Some(PaPaint::from_gradient(gradient.build(self, opacity))),
                Some(Item::RadialGradient(ref gradient)) => Some(PaPaint::from_gradient(gradient.build(self, opacity))),
                r => {
                    dbg!(id, r);
                    None
                }
            }
            _ => None
        }
    }
    fn draw(&self, scene: &mut Scene, path: &Outline) {
        if let Some(ref fill) = self.resolve_paint(&self.fill, self.fill_opacity) {
            let paint_id = scene.push_paint(fill);
            let mut draw_path = DrawPath::new(path.clone().transformed(&self.transform), paint_id);
            draw_path.set_fill_rule(self.fill_rule);
            scene.push_draw_path(draw_path);
        }
        if let Some(ref stroke) = self.resolve_paint(&self.stroke, self.stroke_opacity) {
            let paint_id = scene.push_paint(stroke);
            let mut stroke = OutlineStrokeToFill::new(path, self.stroke_style);
            stroke.offset();
            let path = stroke.into_outline();
            let draw_path = DrawPath::new(path.transformed(&self.transform), paint_id);
            scene.push_draw_path(draw_path);
        }
    }
    fn apply(&self, attrs: &Attrs) -> DrawOptions {
        let mut stroke_style = self.stroke_style;
        if let Some(length) = attrs.stroke_width {
            stroke_style.line_width = length.num as f32;
        }
        DrawOptions {
            transform: self.transform * attrs.transform,
            fill: merge(&attrs.fill, &self.fill),
            fill_rule: attrs.fill_rule.unwrap_or(self.fill_rule),
            fill_opacity: attrs.fill_opacity.unwrap_or(self.fill_opacity),
            stroke: merge(&attrs.stroke, &self.stroke),
            stroke_style,
            stroke_opacity: attrs.stroke_opacity.unwrap_or(self.stroke_opacity),
            #[cfg(feature="debug")]
            debug_font: self.debug_font.clone(),
            .. *self
        }
    }
    fn resolve_length(&self, length: Length) -> f32 {
        let scale = match length.unit {
            LengthUnit::None => 1.0,
            LengthUnit::Cm => self.ctx.dpi * (1.0 / 2.54),
            LengthUnit::Em => unimplemented!(),
            LengthUnit::Ex => unimplemented!(),
            LengthUnit::In => self.ctx.dpi,
            LengthUnit::Mm => self.ctx.dpi * (1.0 / 25.4),
            LengthUnit::Pc => unimplemented!(),
            LengthUnit::Percent => unimplemented!(),
            LengthUnit::Pt => self.ctx.dpi * (1.0 / 75.),
            LengthUnit::Px => 1.0
        };
        length.num as f32 * scale
    }
    fn resolve_point(&self, (x, y): (Length, Length)) -> Vector2F {
        let x = self.resolve_length(x);
        let y = self.resolve_length(y);
        vec2f(x, y)
    }
}

fn merge<T: Clone>(a: &Option<T>, b: &Option<T>) -> Option<T> {
    match (a, b) {
        (_, &Some(ref b)) => Some(b.clone()),
        (&Some(ref a), &None) => Some(a.clone()),
        (&None, &None) => None
    }
}

#[derive(Debug)]
pub enum Item {
    Path(TagPath),
    G(TagG),
    Defs(TagDefs),
    Rect(TagRect),
    Polygon(TagPolygon),
    LinearGradient(TagLinearGradient),
    RadialGradient(TagRadialGradient),
}

impl Item {
    fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        match *self {
            Item::G(ref tag) => tag.compose_to(scene, &options),
            Item::Path(ref tag) => tag.compose_to(scene, &options),
            Item::Rect(ref tag) => tag.compose_to(scene, &options),
            Item::Polygon(ref tag) => tag.compose_to(scene, &options),
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct TagG {
    items: Vec<Arc<Item>>,
    attrs: Attrs,
}
impl TagG {
    pub fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(&self.attrs);
        for item in &self.items {
            item.compose_to(scene, &options);
        }
    }
}
#[derive(Debug)]
pub struct TagDefs {
    items: Vec<Arc<Item>>,
}

fn link(ids: &mut ItemCollection, item: &Arc<Item>) {
    match &**item {
        Item::G(g) => g.items.iter().for_each(|item| link(ids, item)),
        Item::Defs(defs) => defs.items.iter().for_each(|item| link(ids, item)),
        Item::LinearGradient(TagLinearGradient { id: Some(id), .. }) => { ids.insert(id.clone(), item.clone()); }
        Item::RadialGradient(TagRadialGradient { id: Some(id), .. }) => { ids.insert(id.clone(), item.clone()); }
        _ => {}
    }
}


fn parse_node_list<'a, 'i: 'a>(nodes: impl Iterator<Item=Node<'a, 'i>>) -> Result<Vec<Arc<Item>>, Error<'a>> {
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

fn parse_node<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<Option<Item>, Error<'i>> {
    Ok(match node.tag_name().name() {
        "title" | "desc" => None,
        "defs" => Some(Item::Defs(tag_defs(node)?)),
        "path" => Some(Item::Path(TagPath::parse(node)?)),
        "g" => Some(Item::G(tag_g(node)?)),
        "rect" => Some(Item::Rect(TagRect::parse(node)?)),
        "polygon" => Some(Item::Polygon(TagPolygon::parse(node)?)),
        "linearGradient" => Some(Item::LinearGradient(TagLinearGradient::parse(node)?)),
        "radialGradient" => Some(Item::RadialGradient(TagRadialGradient::parse(node)?)),
        tag => return Err(Error::Unimplemented(tag))
    })
}


fn tag_g<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagG, Error<'i>> {
    let attrs = Attrs::parse(node)?;
    let items = parse_node_list(node.children())?;
    Ok(TagG { items, attrs })
}

fn tag_defs<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagDefs, Error<'i>> {
    let items = parse_node_list(node.children())?;
    Ok(TagDefs { items })
}
