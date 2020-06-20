use pathfinder_geometry::{
    vector::Vector2F,
    transform2d::Transform2F,
    rect::RectF,
};
use pathfinder_color::ColorU;
use svgtypes::{TransformListParser, TransformListToken, Length, LengthListParser};
use crate::error::Error;
use std::str::FromStr;

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

#[inline]
pub fn vec(x: f64, y: f64) -> Vector2F {
    Vector2F::new(x as f32, y as f32)
}

pub fn transform_list(value: &str) -> Result<Transform2F, Error> {
    let mut transform = Transform2F::default();
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
    Ok(transform)
}


#[derive(Debug)]
pub struct Rect {
    origin: (Length, Length),
    size: (Length, Length)
}
impl Rect {
    pub fn from_size(width: Length, height: Length) -> Rect {
        Rect {
            origin: (Length::zero(), Length::zero()),
            size: (width, height)
        }
    }

    pub fn as_rectf(&self) -> RectF {
        let (x, y) = self.origin;
        let (w, h) = self.size;
        RectF::new(vec(x.num, y.num), vec(w.num, h.num))
    }

    pub fn parse(s: &str) -> Result<Rect, Error> {
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

pub fn merge<T: Clone>(a: &Option<T>, b: &Option<T>) -> Option<T> {
    match (a, b) {
        (&Some(ref a), _) => Some(a.clone()),
        (&None, &Some(ref b)) => Some(b.clone()),
        (&None, &None) => None
    }
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
