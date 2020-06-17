use crate::prelude::*;

use pathfinder_color::ColorU;
use pathfinder_content::{
    outline::{Outline, ArcDirection, Contour},
    stroke::{OutlineStrokeToFill, StrokeStyle, LineCap, LineJoin},
    fill::{FillRule}
};
use svgtypes::{Length, Color};

#[derive(Debug)]
pub struct Attrs {
    pub clip_path: Option<ClipPathAttr>,
    pub clip_rule: Option<FillRule>,
    pub transform: Transform2F,
    pub fill: Option<Paint>,
    pub fill_rule: Option<FillRule>,
    pub fill_opacity: Option<f32>,
    pub stroke: Option<Paint>,
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
            fill: None,
            fill_rule: Some(FillRule::Winding),
            fill_opacity: Some(1.0),
            stroke: None,
            stroke_width: Some(Length::new_number(1.0)),
            stroke_opacity: Some(1.0),
            display: true,
            filter: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Paint {
    None,
    Inherit,
    CurrentColor,
    Color(Color),
    Ref(String),
}
impl Paint {
    pub fn parse(s: &str) -> Result<Paint, Error> {
        use svgtypes::Paint as SvgPaint;
        Ok(match SvgPaint::from_str(s)? {
            SvgPaint::None => Paint::None,
            SvgPaint::Inherit => Paint::Inherit,
            SvgPaint::CurrentColor => Paint::CurrentColor,
            SvgPaint::Color(color) => Paint::Color(color),
            SvgPaint::FuncIRI(s, _)  => Paint::Ref(s.to_owned()),
            p => {
                dbg!(p);
                return Err(Error::InvalidAttributeValue(s));
            }
        })
    }
    pub fn is_none(&self) -> bool {
        matches!(*self, Paint::None)
    }
}

impl Attrs {
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<Attrs, Error<'i>> {
        let mut attrs = Attrs::default();
        for attr in node.attributes() {
            attrs.parse_entry(attr.name(), attr.value())?;
        }
        println!("attrs: {:?} -> {:?}", node, &attrs);
        Ok(attrs)
    }

    fn parse_entry<'a>(&mut self, key: &'a str, val: &'a str) -> Result<(), Error<'a>> {
        match key {
            "clip-path" => self.clip_path = ClipPathAttr::parse(val)?,
            "clip-rule" => self.clip_rule = fill_rule(val)?,
            "fill" => self.fill = Some(Paint::parse(val)?),
            "fill-opacity" => self.fill_opacity = Some(opacity(val)?),
            "fill-rule" => self.fill_rule = fill_rule(val)?,
            "stroke" => self.stroke = Some(Paint::parse(val)?),
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
        val => return Err(Error::InvalidAttributeValue(val))
    })
}

fn display(s: &str) -> Result<bool, Error> {
    match s {
        "none" => Ok(false),
        "inline" => Ok(true),
        val => Err(Error::InvalidAttributeValue(val))
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
        Err(Error::InvalidAttributeValue(s))
    }
}