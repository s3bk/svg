use crate::prelude::*;

use pathfinder_content::{
    fill::{FillRule}
};
use svgtypes::{Length};
use isolang::Language;

#[derive(Debug, Clone)]
pub struct Attrs {
    pub clip_path: Option<ClipPathAttr>,
    pub clip_rule: Option<FillRule>,
    pub transform: Transform,
    pub opacity: Value<Option<f32>>,
    pub fill: Value<Fill>,
    pub fill_rule: Option<FillRule>,
    pub fill_opacity: Value<Option<f32>>,
    pub stroke: Value<Stroke>,
    pub stroke_width: Value<Option<Length>>,
    pub stroke_opacity: Value<Option<f32>>,
    pub stroke_dasharray: Value<Option<DashArray>>,
    pub stroke_dashoffset: Value<Option<Length>>,
    pub display: bool,
    pub filter: Option<Iri>,
    pub font_size: Value<Option<LengthY>>,
    pub direction: Option<TextFlow>,
    pub lang: Option<Language>,
}

#[derive(Debug, Clone)]
pub struct Fill(pub Option<Paint>);
impl Parse for Fill {
    fn parse(s: &str) -> Result<Self, Error> {
        Ok(Fill(parse_paint(s)?))
    }
}

fn parse_paint(s: &str) -> Result<Option<Paint>, Error> {
    match s {
        "inherit" | "currentColor" | "currentcolor" => Ok(None),
        _ => Paint::parse(s).map(Some)
    }
}

#[derive(Debug, Clone)]
pub struct Stroke(pub Option<Paint>);
impl Parse for Stroke {
    fn parse(s: &str) -> Result<Self, Error> {
        Ok(Stroke(parse_paint(s)?))
    }
}

fn parse_lang_attr(s: &str) -> Option<Language> {
    Language::from_639_3(s).or_else(|| Language::from_639_1(s))
}

impl Parse for Language {
    fn parse(s: &str) -> Result<Self, Error> {
        let lang = s.split("-").next().unwrap().to_ascii_lowercase();
        parse_lang_attr(&lang).ok_or_else(|| Error::InvalidAttributeValue(s.into()))
    }
}

impl Attrs {
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<Attrs, Error> {
        parse!(node => {
            var clip_path ("clip-path"): Option<ClipPathAttr> => ClipPathAttr::parse,
            var clip_rule ("clip-rule"): Option<FillRule>,
            anim transform: Transform,
            anim opacity: Value<Option<f32>>,
            anim fill: Value<Fill> = Value::new(Fill(None)),
            var fill_rule ("fill-rule"): Option<FillRule> = Some(FillRule::Winding) => inherit(FillRule::parse),
            anim fill_opacity ("fill-opacity"): Value<Option<f32>>,
            anim stroke: Value<Stroke> = Value::new(Stroke(None)),
            anim stroke_width ("stroke-width"): Value<Option<Length>>,
            anim stroke_opacity ("stroke-opacity"): Value<Option<f32>>,
            anim stroke_dasharray ("stroke-dasharray"): Value<Option<DashArray>>,
            anim stroke_dashoffset ("stroke-dashoffset"): Value<Option<Length>>,
            var display: bool = true => parse_display,
            var filter: Option<Iri>,
            anim font_size ("font-size"): Value<Option<LengthY>>,
            var direction: Option<TextFlow>,
            var lang: Option<Language>,
        });
        Ok(Attrs {
            clip_path,
            clip_rule,
            transform,
            opacity,
            fill,
            fill_rule,
            fill_opacity,
            stroke,
            stroke_width,
            stroke_opacity,
            stroke_dasharray,
            stroke_dashoffset,
            display,
            filter,
            font_size,
            direction,
            lang,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DashArray(pub Vec<Length>);
impl Parse for DashArray {
    fn parse(s: &str) -> Result<DashArray, Error> {
        let lengths = Vec::<Length>::parse(s)?;
        let lengths = if lengths.len() % 2 == 0 {
            lengths
        } else {
            let mut v = Vec::with_capacity(2 * lengths.len());
            v.extend_from_slice(&lengths);
            v.extend_from_slice(&lengths);
            v
        };
        Ok(DashArray(lengths))
    }
}

impl Parse for FillRule {
    fn parse(s: &str) -> Result<FillRule, Error> {
        Ok(match s {
            "nonzero" => FillRule::Winding,
            "evenodd" => FillRule::EvenOdd,
            val => return Err(Error::InvalidAttributeValue(val.into()))
        })
    }
}

fn parse_display(s: &str) -> Result<bool, Error> {
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

#[derive(Debug, Copy, Clone)]
pub enum TextFlow {
    LeftToRight,
    RightToLeft
}

impl Parse for TextFlow {
    fn parse(s: &str) -> Result<TextFlow, Error> {
        Ok(match s {
            "ltr" => TextFlow::LeftToRight,
            "rtl" => TextFlow::RightToLeft,
            val => return Err(Error::InvalidAttributeValue(val.into()))
        })
    }
}