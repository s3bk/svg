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

use svgtypes::{Length, LengthListParser, Color};
use std::sync::Arc;

mod prelude;

mod util;
use util::*;

mod path;
use path::*;

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

mod filter;
use filter::*;

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
    pub fn parse<'a>(doc: &'a Document) -> Result<Svg, Error<'a>> {
        let root = doc.root_element();
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

    clip_path: ClipPathAttr,
    clip_rule: FillRule,
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
            transform: Transform2F::from_scale(10.),
            clip_path: ClipPathAttr::None,
            clip_rule: FillRule::EvenOdd,
        }
    }
    pub fn bounds(&self, rect: RectF) -> Option<RectF> {
        let has_fill = matches!(*self,
            DrawOptions { fill: Some(ref paint), fill_opacity, .. }
            if !paint.is_none() && fill_opacity > 0.);
        let has_stroke = matches!(*self,
            DrawOptions { stroke: Some(ref paint), stroke_opacity, .. }
            if !paint.is_none() && stroke_opacity > 0.
        );

        if has_stroke {
            Some(self.transform * rect.dilate(self.stroke_style.line_width))
        } else if has_fill {
            Some(self.transform * rect)
        } else {
            None
        }
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
    fn debug_outline(&self, scene: &mut Scene, path: &Outline, color: ColorU) {
        dbg!(path);
        let paint_id = scene.push_paint(&PaPaint::from_color(color));
        scene.push_draw_path(DrawPath::new(path.clone(), paint_id));
    }
    fn clip_path_id(&self, scene: &mut Scene) -> Option<ClipPathId> {
        if let ClipPathAttr::Ref(ref id) = self.clip_path {
            if let Some(Item::ClipPath(TagClipPath { outline, .. })) = self.ctx.resolve(id).map(|t| &**t) {
                let outline = outline.clone().transformed(&self.transform);
                println!("clip path: {:?}", outline);
                //self.debug_outline(scene, &outline, ColorU::new(0, 255, 0, 50));

                let mut clip_path = ClipPath::new(outline);
                clip_path.set_fill_rule(self.clip_rule);
                return Some(scene.push_clip_path(clip_path));
            } else {
                println!("clip path missing: {}", id);
            }
        }
        None
    }
    fn draw(&self, scene: &mut Scene, path: &Outline) {
        let clip_path_id = self.clip_path_id(scene);
        
        if let Some(ref fill) = self.resolve_paint(&self.fill, self.fill_opacity) {
            let outline = path.clone().transformed(&self.transform);
            println!("draw {:?}", outline);
            let paint_id = scene.push_paint(fill);
            let mut draw_path = DrawPath::new(outline, paint_id);
            draw_path.set_fill_rule(self.fill_rule);
            draw_path.set_clip_path(clip_path_id);
            scene.push_draw_path(draw_path);
        }
        if let Some(ref stroke) = self.resolve_paint(&self.stroke, self.stroke_opacity) {
            if self.stroke_style.line_width > 0. {
                let paint_id = scene.push_paint(stroke);
                let mut stroke = OutlineStrokeToFill::new(path, self.stroke_style);
                stroke.offset();
                let path = stroke.into_outline();
                let mut draw_path = DrawPath::new(path.transformed(&self.transform), paint_id);
                draw_path.set_clip_path(clip_path_id);
                scene.push_draw_path(draw_path);
            }
        }
    }
    fn apply(&self, attrs: &Attrs) -> DrawOptions {
        let mut stroke_style = self.stroke_style;
        if let Some(length) = attrs.stroke_width {
            stroke_style.line_width = length.num as f32;
        }
        DrawOptions {
            clip_path: attrs.clip_path.clone().unwrap_or_else(|| self.clip_path.clone()),
            clip_rule: attrs.clip_rule.unwrap_or(self.clip_rule),
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


#[derive(Debug)]
pub enum Item {
    Path(TagPath),
    G(TagG),
    Defs(TagDefs),
    Rect(TagRect),
    Polygon(TagPolygon),
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
            _ => {}
        }
    }
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        match *self {
            Item::G(ref tag) => tag.bounds(&options),
            Item::Path(ref tag) => tag.bounds(&options),
            Item::Rect(ref tag) => tag.bounds(&options),
            Item::Polygon(ref tag) => tag.bounds(&options),
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

        dbg!(&self.attrs, options.transform);
        let options = options.apply(&self.attrs);

        if let Some(ref filter_id) = self.attrs.filter {
            let bounds = match max_bounds(self.items.iter().flat_map(|item| dbg!(item.bounds(&options)))) {
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
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagG, Error<'i>> {
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
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagDefs, Error<'i>> {
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
        Item::Filter(TagFilter { id: Some(id), .. }) => {
             ids.insert(id.clone(), item.clone());
        }
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
    println!("<{:?}:{} id={:?}, ...>", node.tag_name().namespace(), node.tag_name().name(), node.attribute("id"));
    Ok(match node.tag_name().name() {
        "title" | "desc" | "metadata" => None,
        "defs" => Some(Item::Defs(TagDefs::parse(node)?)),
        "path" => Some(Item::Path(TagPath::parse(node)?)),
        "g" => Some(Item::G(TagG::parse(node)?)),
        "rect" => Some(Item::Rect(TagRect::parse(node)?)),
        "polygon" => Some(Item::Polygon(TagPolygon::parse(node)?)),
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

