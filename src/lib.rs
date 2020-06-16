#[macro_use] extern crate lazy_static;

use roxmltree::{Document, Node, Error as XmlError, NodeType};
use pathfinder_geometry::{
    vector::Vector2F,
    transform2d::{Matrix2x2F, Transform2F},
    rect::RectF,
    line_segment::LineSegment2F,
};
use pathfinder_content::{
    outline::{Outline, ArcDirection, Contour},
    stroke::{OutlineStrokeToFill, StrokeStyle, LineCap, LineJoin}
};
use pathfinder_renderer::{
    scene::{Scene, DrawPath},
    paint::Paint,
};
use pathfinder_color::ColorU;
use std::sync::Arc;

use svgtypes::{Error as SvgError, Length, LengthListParser};

mod path;
use path::*;

mod debug;
use debug::*;

mod text;
use text::*;

#[derive(Debug)]
pub struct Svg {
    items: Vec<Item>,
    view_box: Option<Rect>,
}
impl Svg {
    pub fn compose(&self) -> Scene {
        let mut scene = Scene::new();
        if let Some(ref r) = self.view_box {
            scene.set_view_box(dbg!(r.as_rectf()));
        }
        self.compose_to(&mut scene, Transform2F::default());
        scene
    }
    pub fn compose_to(&self, scene: &mut Scene, tr: Transform2F) {
        let options = DrawOptions::new(tr);
        for item in &self.items {
            item.compose_to(scene, &options);
        }
    }
}

#[derive(Clone)]
struct DrawOptions {
    fill: Option<Paint>,
    stroke: Option<Paint>,
    stroke_style: StrokeStyle,
    transform: Transform2F,
    debug_font: Arc<FontCollection>,
}
impl DrawOptions {
    fn new(transform: Transform2F) -> DrawOptions {
        DrawOptions {
            fill: None,
            stroke: None,
            stroke_style: StrokeStyle {
                line_width: 1.0,
                line_cap: LineCap::Butt,
                line_join: LineJoin::Bevel,
            },
            transform,
            debug_font: Arc::new(FontCollection::debug()),
        }
    }
    fn draw(&self, scene: &mut Scene, path: &Outline) {
        if let Some(ref fill) = self.fill {
            let paint_id = scene.push_paint(fill);
            let draw_path = DrawPath::new(path.clone().transformed(&self.transform), paint_id);
            scene.push_draw_path(draw_path);
        }
        if let Some(ref stroke) = self.stroke {
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
            stroke: merge(&attrs.stroke, &self.stroke),
            stroke_style,
            debug_font: self.debug_font.clone()
        }
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
pub enum ItemKind {
    Path(Outline),
    G(Vec<Item>),
}

#[derive(Debug)]
pub struct Item {
    kind: ItemKind,
    attrs: Attrs,
    debug: DebugInfo,
}

impl Item {
    fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let mut options = options.apply(&self.attrs);
        match self.kind {
            ItemKind::G(ref items) => {
                for item in items {
                    item.compose_to(scene, &options);
                }
            }
            ItemKind::Path(ref outline) => {
                options.draw(scene, outline);

                options.fill = Some(Paint::black());
                options.stroke = None;
                self.debug.draw(scene, &options);
            }
        }
    }
}
#[derive(Debug)]
pub struct Attrs {
    pub transform: Transform2F,
    pub fill: Option<Paint>,
    pub stroke: Option<Paint>,
    pub stroke_width: Option<Length>,
}

#[derive(Debug)]
pub enum Error {
    Xml(XmlError),
    Svg(SvgError),
    TooShort,
    Unimplemented
}
impl From<XmlError> for Error {
    fn from(e: XmlError) -> Self {
        Error::Xml(e)
    }
}
impl From<SvgError> for Error {
    fn from(e: SvgError) -> Self {
        Error::Svg(e)
    }
}

pub fn parse(data: &str) -> Result<Svg, Error> {
    let doc = Document::parse(data)?;
    dbg!(&doc);
    let root = doc.root_element();
    dbg!(root.tag_name());
    assert!(root.has_tag_name("svg"));
    let view_box = root.attribute("viewBox").map(|v| parse_rect(v)).transpose()?;

    let items = parse_list(root.children())?;

    Ok(Svg { items, view_box })
}

fn parse_list<'a, 'i: 'a>(nodes: impl Iterator<Item=Node<'a, 'i>>) -> Result<Vec<Item>, Error> {
    let mut items = Vec::new();
    for node in nodes {
        match node.node_type() {
            NodeType::Element => items.push(parse_node(&node)?),
            _ => {}
        }
    }
    Ok(items)
}

#[derive(Debug)]
struct Rect {
    origin: (Length, Length),
    size: (Length, Length)
}
impl Rect {
    fn as_rectf(&self) -> RectF {
        let (x, y) = self.origin;
        let (w, h) = self.size;
        RectF::new(vec(x.num, y.num), vec(w.num, h.num))
    }
}

fn parse_rect(s: &str) -> Result<Rect, Error> {
    let mut p = LengthListParser::from(s);
    let x = p.next().ok_or(Error::TooShort)??;
    let y = p.next().ok_or(Error::TooShort)??;
    let w = p.next().ok_or(Error::TooShort)??;
    let h = p.next().ok_or(Error::TooShort)??;
    Ok(Rect {
        origin: (x, y),
        size: (w, h)
    })
}

fn parse_node(node: &Node) -> Result<Item, Error> {
    match node.tag_name().name() {
        "path" => parse_path(node),
        "g" => parse_g(node),
        _ => Err(Error::Unimplemented)
    }
}

fn parse_g(node: &Node) -> Result<Item, Error> {
    let attrs = parse_attrs(node)?;
    let children = parse_list(node.children())?;
    Ok(Item { kind: ItemKind::G(children), attrs, debug: DebugInfo::new() })
}

fn parse_attrs(node: &Node) -> Result<Attrs, Error> {
    use svgtypes::{TransformListParser, TransformListToken};
    let mut transform = Transform2F::default();

    if let Some(value) = node.attribute("transform") {
        for op in TransformListParser::from(value) {
            let tr = match op? {
                TransformListToken::Matrix { a, b, c, d, e, f } => Transform2F::row_major(a as f32, c as f32, e as f32, b as f32, d as f32, f as f32),
                TransformListToken::Translate { tx, ty } => Transform2F::from_translation(vec(tx, ty)),
                TransformListToken::Scale { sx, sy } => Transform2F::from_scale(vec(sx, sy)),
                TransformListToken::Rotate { angle } => Transform2F::from_rotation(angle as f32),
                TransformListToken::SkewX { angle } => Transform2F::row_major(1.0, angle.tan() as f32, 0.0, 0.0, 1.0, 0.0),
                TransformListToken::SkewY { angle} => Transform2F::row_major(1.0, 0.0, 0.0, angle.tan() as f32, 1.0, 0.0),
            };
            transform = transform * tr;
        }
    }

    let fill = match node.attribute("fill") {
        Some(val) => parse_paint(val)?,
        _ => None
    };
    let stroke = match node.attribute("stroke") {
        Some(val) => parse_paint(val)?,
        _ => None
    };
    let stroke_width = node.attribute("stroke-width").map(|val| val.parse()).transpose()?;
    Ok(Attrs { transform, fill, stroke, stroke_width })
}

fn parse_paint(value: &str) -> Result<Option<Paint>, Error> {
    use svgtypes::{Color};
    Ok(match svgtypes::Paint::from_str(value)? {
        svgtypes::Paint::Color(Color { red, green, blue }) => Some(Paint::from_color(ColorU::new(red, green, blue, 255))),
        svgtypes::Paint::None => None,
        _ => None
    })
}

#[inline]
fn vec(x: f64, y: f64) -> Vector2F {
    Vector2F::new(x as f32, y as f32)
}