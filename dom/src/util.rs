use pathfinder_geometry::{
    vector::{Vector2F, vec2f},
    transform2d::Transform2F,
    rect::RectF,
};
use svgtypes::{TransformListParser, TransformListToken, Length, LengthListParser};
use crate::error::Error;
use std::str::FromStr;
use roxmltree::Node;

#[cfg(feature="profile")]
#[macro_export]
macro_rules! timed {
    ($label:expr, { $($t:tt)* }) => (
        let t0 = ::std::time::Instant::now();
        let r = $($t)*;
        info!("{}: {:?}", $label, t0.elapsed());
        r
    )
}

#[cfg(not(feature="profile"))]
#[macro_export]
macro_rules! timed {
    ($label:expr, { $($t:tt)* }) => (
        $($t)*
    )
}

#[macro_export]
macro_rules! get_or_return {
    ($opt:expr) => (
        match $opt {
            Some(val) => val,
            None => return
        }
    );
    ($opt:expr, $msg:tt $(,$args:tt)*) => (
        match $opt {
            Some(val) => val,
            None => {
                println!($msg $(,$args)*);
                return;
            }
        }
    );
}
#[macro_export]
macro_rules! get_ref_or_return {
    ($opt:expr) => (
        match $opt {
            Some(ref val) => val,
            None => return
        }
    );
    ($opt:expr, $msg:tt $(,$args:tt)*) => (
        match $opt {
            Some(ref val) => val,
            None => {
                println!($msg $(,$args)*);
                return;
            }
        }
    );
}

const DEG_TO_RAD: f32 = std::f32::consts::PI / 180.;

#[inline]
pub fn vec(x: f64, y: f64) -> Vector2F {
    Vector2F::new(x as f32, y as f32)
}

pub fn skew_x(angle: f32) -> Transform2F {
    Transform2F::row_major(1.0, angle.tan() as f32, 0.0, 0.0, 1.0, 0.0)
}
pub fn skew_y(angle: f32) -> Transform2F {
    Transform2F::row_major(1.0, 0.0, 0.0, angle.tan() as f32, 1.0, 0.0)
}

pub fn deg2rad(deg: f32) -> f32 {
    deg as f32 * DEG_TO_RAD
}

pub fn transform_list(value: &str) -> Result<Transform2F, Error> {
    let mut transform = Transform2F::default();
    for op in TransformListParser::from(value) {
        let tr = match op? {
            TransformListToken::Matrix { a, b, c, d, e, f } => Transform2F::row_major(a as f32, c as f32, e as f32, b as f32, d as f32, f as f32),
            TransformListToken::Translate { tx, ty } => Transform2F::from_translation(vec(tx, ty)),
            TransformListToken::Scale { sx, sy } => Transform2F::from_scale(vec(sx, sy)),
            TransformListToken::Rotate { angle } => Transform2F::from_rotation(deg2rad(angle as f32)),
            TransformListToken::SkewX { angle } => skew_x(deg2rad(angle as f32)),
            TransformListToken::SkewY { angle} => skew_y(deg2rad(angle as f32)),
        };
        transform = transform * tr;
    }
    Ok(transform)
}

#[derive(Copy, Clone, Debug, Default)]
pub struct LengthX(pub Length);
#[derive(Copy, Clone, Debug, Default)]
pub struct LengthY(pub Length);

#[derive(Copy, Clone, Debug)]
pub struct Vector(pub Length, pub Length);
impl Vector {
    pub fn has_area(&self) -> bool {
        self.0.num != 0.0 && self.1.num != 0.0
    }
}

#[derive(Debug)]
pub struct Rect {
    pub x: Length,
    pub y: Length,
    pub width: Length,
    pub height: Length
}
impl Rect {
    pub fn from_size(width: Length, height: Length) -> Rect {
        Rect {
            x: Length::zero(),
            y: Length::zero(),
            width,
            height
        }
    }

    pub fn origin(&self) -> Vector {
        Vector(self.x, self.y)
    }
    pub fn size(&self) -> Vector {
        Vector(self.width, self.height)
    }

    pub fn parse(s: &str) -> Result<Rect, Error> {
        let mut p = LengthListParser::from(s);
        Ok(Rect {
            x: p.next().ok_or(Error::TooShort)??,
            y: p.next().ok_or(Error::TooShort)??,
            width: p.next().ok_or(Error::TooShort)??,
            height: p.next().ok_or(Error::TooShort)??,
        })
    }
}

pub fn inherit<T>(f: impl Fn(&str) -> Result<T, Error>) -> impl Fn(&str) -> Result<Option<T>, Error> {
    move |s | match s {
        "inherit" => Ok(None),
        _ => Ok(Some(f(s)?))
    }
}

pub fn opacity(s: &str) -> Result<f32, Error> {
    let val: f32 = s.parse().map_err(|e| Error::InvalidAttributeValue(s.into()))?;
    Ok(val.min(1.0).max(0.0))
}

fn pair<I: Iterator>(mut iter: I) -> Option<(I::Item, I::Item)> {
    match (iter.next(), iter.next()) {
        (Some(a), Some(b)) => Some((a, b)),
        _ => None
    }
}

pub fn style_list(s: &str) -> impl Iterator<Item=(&str, &str)> + '_ {
    s.split(";").flat_map(|s| pair(s.splitn(2, ":"))).map(|(a, b)| (a.trim(), b.trim()))
}

pub fn max_bounds(mut iter: impl Iterator<Item=RectF>) -> Option<RectF> {
    if let Some(mut b) = iter.next() {
        for r in iter {
            b = b.union_rect(r);
        }
        Some(b)
    } else {
        None
    }
}

pub fn length(s: &str) -> Result<Length, Error> {
    Ok(Length::from_str(s)?)
}

pub fn href(node: &Node) -> Option<String> {
    let xlink = node.lookup_namespace_uri(Some("xlink")).unwrap_or_default();
    node.attribute((xlink, "href")).map(|s| s.to_owned())
}

pub struct Iri(String);
impl Parse for Iri {
    fn parse(s: &str) -> Result<Self, Error> {
        match crate::parser::func_iri(s) {
            Ok(("", link)) => Ok(Iri(link.into())),
            _ => Err(Error::InvalidAttributeValue(s.into()))
        }
    }
}

pub trait Parse: Sized {
    fn parse(s: &str) -> Result<Self, Error>;
}

impl Parse for f32 {
    fn parse(s: &str) -> Result<Self, Error> {
        Ok(f32::from_str(s)?)
    }
}
impl Parse for Length {
    fn parse(s: &str) -> Result<Self, Error> {
        Ok(Length::from_str(s)?)
    }
}
impl Parse for LengthX {
    fn parse(s: &str) -> Result<Self, Error> {
        Length::parse(s).map(LengthX)
    }
}
impl Parse for LengthY {
    fn parse(s: &str) -> Result<Self, Error> {
        Length::parse(s).map(LengthY)
    }
}

pub fn get_attr<'a, 'i>(node: &Node<'a, 'i>, attr: &str) -> Result<&'a str, Error> {
    match node.attribute(attr) {
        Some(val) => Ok(val),
        None => Err(Error::MissingAttribute(attr.into()))
    }
}

pub fn parse_attr<'a, 'i, T: Parse>(node: &Node<'a, 'i>, attr: &str) -> Result<T, Error> {
    match node.attribute(attr) {
        Some(val) => T::parse(val),
        None => Err(Error::MissingAttribute(attr.into()))
    }
}

pub fn parse_attr_or<'a, 'i, T: Parse>(node: &Node<'a, 'i>, attr: &str, default: T) -> Result<T, Error> {
    match node.attribute(attr) {
        Some(val) => T::parse(val),
        None => Ok(default)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Axis {
    X,
    Y
}