use crate::prelude::*;
use crate::animate::*;

use pathfinder_content::{
    fill::{FillRule}
};
use svgtypes::{Length, Color};

#[derive(Debug)]
pub struct Attrs {
    pub clip_path: Option<ClipPathAttr>,
    pub clip_rule: Option<FillRule>,
    pub transform: Transform2F,
    pub opacity: Option<f32>,
    pub fill: Value<Fill>,
    pub fill_rule: Option<FillRule>,
    pub fill_opacity: Option<f32>,
    pub stroke: Value<Stroke>,
    pub stroke_width: Option<Length>,
    pub stroke_opacity: Option<f32>,
    pub display: bool,
    pub filter: Option<String>,
}
impl Default for Attrs {
    fn default() -> Attrs {
        Attrs {
            clip_path: Some(ClipPathAttr::None),
            clip_rule: Some(FillRule::Winding),
            transform: Transform2F::default(),
            opacity: Some(1.0),
            fill: Value::new(Fill(None)),
            fill_rule: Some(FillRule::Winding),
            fill_opacity: Some(1.0),
            stroke: Value::new(Stroke(None)),
            stroke_width: Some(Length::new_number(1.0)),
            stroke_opacity: Some(1.0),
            display: true,
            filter: None,
        }
    }
}


#[derive(Debug, Clone)]
pub struct Fill(Option<Paint>);
impl Resolve for Fill {
    type Output = Paint;
    fn resolve(&self, options: &DrawOptions) -> Self::Output {
        self.0.clone().unwrap_or_else(|| options.fill.clone())
    }
}
impl Parse for Fill {
    fn parse(s: &str) -> Result<Self, Error> {
        Ok(Fill((inherit(Paint::parse))(s)?))
    }
}
impl Interpolate for Fill {
    fn linear(from: Self, to: Self, x: f32) -> Self {
        Fill(Interpolate::linear(from.0, to.0, x))
    }
    fn add(a: Self, b: Self) -> Self {
        Fill(Interpolate::add(a.0, b.0))
    }
}

#[derive(Debug, Clone)]
pub struct Stroke(Option<Paint>);
impl Resolve for Stroke {
    type Output = Paint;
    fn resolve(&self, options: &DrawOptions) -> Self::Output {
        self.0.clone().unwrap_or_else(|| options.stroke.clone())
    }
}
impl Parse for Stroke {
    fn parse(s: &str) -> Result<Self, Error> {
        Ok(Stroke((inherit(Paint::parse))(s)?))
    }
}
impl Interpolate for Stroke {
    fn linear(from: Self, to: Self, x: f32) -> Self {
        Stroke(Interpolate::linear(from.0, to.0, x))
    }
    fn add(a: Self, b: Self) -> Self {
        Stroke(Interpolate::add(a.0, b.0))
    }
}

impl Attrs {
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<Attrs, Error> {
        let mut attrs = Attrs::default();
        for attr in node.attributes() {
            attrs.parse_entry(attr.name(), attr.value())?;
        }
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "animate" | "animateColor" => attrs.parse_animate(&n)?,
                _ => {}
            }
        }
        Ok(attrs)
    }

    fn parse_animate(&mut self, node: &Node) -> Result<(), Error> {
        let key = node.attribute("attributeName").unwrap();
        match key {
            "fill" => self.fill.parse_animate_node(node),
            "stroke" => self.stroke.parse_animate_node(node),
            _ => Ok(())
        }
    }

    fn parse_entry(&mut self, key: &str, val: &str) -> Result<(), Error> {
        match key {
            "clip-path" => self.clip_path = ClipPathAttr::parse(val)?,
            "clip-rule" => self.clip_rule = fill_rule(val)?,
            "opacity" => self.opacity = (inherit(opacity))(val)?,
            "fill" => self.fill = Value::new(Fill((inherit(Paint::parse))(val)?)),
            "fill-opacity" => self.fill_opacity = Some(opacity(val)?),
            "fill-rule" => self.fill_rule = fill_rule(val)?,
            "stroke" => self.stroke = Value::new(Stroke((inherit(Paint::parse))(val)?)),
            "stroke-width" => self.stroke_width = Some(val.parse()?),
            "stroke-linecap" => {},
            "stroke-linejoin" => {},
            "stroke-miterlimit" => {},
            "stroke-opacity" => self.stroke_opacity = Some(opacity(val)?),
            "stroke-dasharray" => {},
            "transform" => self.transform = transform_list(val)?,
            "paint-order" => {},
            "display" => self.display = display(val)?,
            "filter" => self.filter = Some(iri(val)?),
            "style" => {
                for (key, val) in style_list(val) {
                    self.parse_entry(key, val)?;
                }
            }
            _ => {
                debug!("unhandled {}={}", key, val);
            }
        }
        Ok(())
    }
}

fn fill_rule(s: &str) -> Result<Option<FillRule>, Error> {
    Ok(match s {
        "nonzero" => Some(FillRule::Winding),
        "evenodd" => Some(FillRule::EvenOdd),
        "inherit" => None,
        val => return Err(Error::InvalidAttributeValue(val.into()))
    })
}

fn display(s: &str) -> Result<bool, Error> {
    match s {
        "none" => Ok(false),
        "inline" => Ok(true),
        val => Err(Error::InvalidAttributeValue(val.into()))
    }
}

#[derive(Debug, Clone)]
pub enum ClipPathAttr {
    None,
    Ref(String)
}
impl ClipPathAttr {
    pub fn parse(s: &str) -> Result<Option<ClipPathAttr>, Error> {
        match s {
            "none" => Ok(Some(ClipPathAttr::None)),
            "inherit" => Ok(None),
            _ => Ok(Some(ClipPathAttr::Ref(iri(s)?)))
        }
    }
}

fn iri(s: &str) -> Result<String, Error> {
    if s.starts_with("url(#") && s.ends_with(")") {
        Ok(s[5 .. s.len() - 1].to_owned())
    } else {
        Err(Error::InvalidAttributeValue(s.into()))
    }
}