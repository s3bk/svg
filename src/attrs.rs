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
    pub transform: Transform2F,
    pub fill: Option<Paint>,
    pub fill_rule: Option<FillRule>,
    pub fill_opacity: Option<f32>,
    pub stroke: Option<Paint>,
    pub stroke_width: Option<Length>,
    pub stroke_opacity: Option<f32>,
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
}

impl Attrs {
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<Attrs, Error<'i>> {
        let transform = node.attribute("transform").map(transform_list).transpose()?.unwrap_or_default();

        let fill = node.attribute("fill").map(Paint::parse).transpose()?;
        let fill_opacity = node.attribute("fill-opacity").map(opacity).transpose()?;
        let stroke = node.attribute("stroke").map(Paint::parse).transpose()?;
        let fill_rule = match node.attribute("fill-rule") {
            Some("nonzero") => Some(FillRule::Winding),
            Some("evenodd") => Some(FillRule::EvenOdd),
            Some("inherit") => None,
            None => Some(FillRule::Winding),
            Some(val) => return Err(Error::InvalidAttributeValue(val))
        };
        let stroke_width = node.attribute("stroke-width").map(|val| val.parse()).transpose()?;
        let stroke_opacity = node.attribute("stroke-opacity").map(opacity).transpose()?;
        Ok(Attrs { transform, fill, fill_opacity, stroke, stroke_width, fill_rule, stroke_opacity })
    }
}
