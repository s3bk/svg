use std::ops::{Add, Sub, Mul};
use std::fmt::Debug;
use pathfinder_content::outline::Contour;
use crate::prelude::*;
use crate::parser::{number_list_4, one_or_two_numbers, one_or_three_numbers};

#[derive(Debug, Clone)]
pub struct Animate<T> {
    pub timing: Timing,
    pub mode: AnimationMode<T>,
    pub fill: AnimationFill,
    pub calc_mode: CalcMode,
    pub additive: Additive,
}
impl<T> Animate<T> where T: Parse + Clone {
    pub fn parse_animate(node: &Node, value: &T) -> Result<Self, Error> {
        let timing = Timing::parse_node(node)?;
        let calc_mode = parse_attr_or(node, "calcMode", CalcMode::Linear)?;
        let mode = AnimationMode::parse_node(node, value, calc_mode)?;
        let fill = parse_attr_or(node, "fill", AnimationFill::Remove)?;
        let default_additive = match mode {
            AnimationMode::Absolute { .. } | AnimationMode::Values { .. } => Additive::Replace,
            AnimationMode::Relative { .. } => Additive::Sum
        };
        let additive = parse_attr_or(node, "additive", default_additive)?;

        Ok(Animate {
            timing,
            mode,
            fill,
            calc_mode,
            additive,
        })
    }
}
impl<T> Animate<T> where T: Parse + Clone + Default {
    fn parse_animate_default(node: &Node) -> Result<Self, Error> {
        Self::parse_animate(node, &T::default())
    }
}

#[derive(Debug, Clone)]
pub struct Timing {
    pub begin: Time,
    pub scale: f32,
   //repeat_until: Time,
}
impl ParseNode for Timing {
    fn parse_node(node: &Node) -> Result<Timing, Error> {
        let begin = parse_attr_or(node, "begin", Time(0.0))?;
        let duration: Time = parse_attr(node, "dur")?;
        Ok(Timing { begin, scale: 1.0 / duration.seconds() })
    }
}
pub struct AnimateMotion {
    pub path: Contour,
    pub path_len: f32,
    pub timing: Timing,
}


#[derive(Debug, Copy, Clone)]
pub enum CalcMode {
    Discrete,
    Linear,
    Paced,
    Spline
}
impl Parse for CalcMode {
    fn parse(s: &str) -> Result<Self, Error> {
        match s {
            "discrete" => Ok(CalcMode::Discrete),
            "linear" => Ok(CalcMode::Linear),
            "paced" => Ok(CalcMode::Paced),
            "spline" => Ok(CalcMode::Spline),
            _ => Err(Error::InvalidAttributeValue(s.into()))
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnitSpline(Vector2F, Vector2F);
impl UnitSpline {
    pub fn at(&self, t: f32) -> Vector2F {
        let u = Vector2F::splat(1.0 - t);
        let t = Vector2F::splat(t);
        let lerp = |a, b| a * u + b * t;

        let p0 = vec2f(0.0, 0.0);
        let p1 = self.0;
        let p2 = self.1;
        let p3 = vec2f(1.0, 1.0);

        let p01 = lerp(p0, p1);
        let p12 = lerp(p1, p2);
        let p23 = lerp(p2, p3);
        let p012 = lerp(p01, p12);
        let p123 = lerp(p12, p23);
        let p0123 = lerp(p012, p123);

        p0123
/*
        p01 = t a
        p12 = u a + t b
        p23 = u b + t

        p012 = 2 t u a + t² b
        p123 = u² a + 2 t u b + t²
        
        p0123 = 3 t u² a + 3 t² u b + t³
         = 3 t (t² + 1 - 2t) a + 3 t² (1 - t) b + t³
         = 3 a (t³ - 2t² + t) - 3 b (t³ - t²) + t³
         = t³ (3a - 3b + 1) + t² (-3a -3b) + t (3a)

*/
    }
    pub fn y_for_x(&self, x: f32) -> f32 {
        let mut low = 0.0;
        let mut high = 1.0;
        let mut f_high = self.at(high);
        let mut f_low = self.at(low);
        for _ in 0 .. 5 {
            let mid = (low + high) * 0.5;
            let p = self.at(mid);
            if x < p.x() {
                high = mid;
                f_high = p;
            } else {
                low = mid;
                f_low = p;
            }
        }

        let delta = f_high - f_low;
        let p = if delta.x() < 1e-5 {
            (f_high + f_low) * 0.5
        } else {
            f_low + delta * ((x - f_low.x()) * (1.0 / delta.x()))
        };

        debug!("y_for_x({}) -> (x={}, y={})", x, p.x(), p.y());
        p.y()
    }
}

impl Timing {
    pub fn pos(&self, t: Time) -> f32 {
        (t - self.begin).seconds() * self.scale
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Time(f64);
impl Sub for Time {
    type Output = Time;
    fn sub(self, rhs: Time) -> Time {
        Time(self.0 - rhs.0)
    }
}
impl Time {
    pub fn from_seconds(seconds: f64) -> Time {
        Time(seconds)
    }
    pub fn seconds(self) -> f32 {
        self.0 as f32
    }
    pub fn start() -> Time {
        Time(0.0)
    }
}
impl Parse for Time {
    fn parse(s: &str) -> Result<Time, Error> {
        assert!(s.ends_with("s"));
        let seconds: f64 = s[.. s.len() - 1].parse().unwrap();
        Ok(Time(seconds))
    }
}


#[derive(Clone, Debug, Default)]
pub struct Translation(pub Vector2F);
impl Into<Transform2F> for Translation {
    fn into(self) -> Transform2F {
        Transform2F::from_translation(self.0)
    }
}

#[derive(Clone, Debug)]
pub struct Scale(pub Vector2F);
impl Into<Transform2F> for Scale {
    fn into(self) -> Transform2F {
        Transform2F::from_scale(self.0)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Rotation(pub f32, pub Vector2F);
impl Into<Transform2F> for Rotation {
    fn into(self) -> Transform2F {
        Transform2F::from_translation(self.1) * Transform2F::from_rotation(self.0) * Transform2F::from_translation(-self.1)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SkewX(pub f32);
impl Into<Transform2F> for SkewX {
    fn into(self) -> Transform2F {
        skew_x(self.0)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SkewY(pub f32);
impl Into<Transform2F> for SkewY {
    fn into(self) -> Transform2F {
        skew_y(self.0)
    }
}

impl Parse for Translation {
    fn parse(s: &str) -> Result<Self, Error> {
        let (x, y) = one_or_two_numbers(s)?;
        Ok(Translation(vec2f(x, y.unwrap_or(0.0))))
    }
}
impl Parse for Scale {
    fn parse(s: &str) -> Result<Self, Error> {
        let (x, y) = one_or_two_numbers(s)?;
        Ok(Scale(vec2f(x, y.unwrap_or(x))))
    }
}
impl Default for Scale {
    fn default() -> Self {
        Scale(vec2f(1.0, 1.0))
    }
}

impl Parse for Rotation {
    fn parse(s: &str) -> Result<Self, Error> {
        let (deg, c) = one_or_three_numbers(s)?;
        let center = c.map(|(x, y)| vec2f(x, y)).unwrap_or_default();
        Ok(Rotation(deg2rad(deg), center))
    }
}
impl Parse for SkewX {
    fn parse(s: &str) -> Result<Self, Error> {
        Ok(SkewX(f32::parse(s)?))
    }
}
impl Parse for SkewY {
    fn parse(s: &str) -> Result<Self, Error> {
        Ok(SkewY(f32::parse(s)?))
    }
}

#[derive(Clone, Debug)]
pub enum TransformAnimate {
    Translate(Animate<Translation>),
    Scale(Animate<Scale>),
    Rotate(Animate<Rotation>),
    SkewX(Animate<SkewX>),
    SkewY(Animate<SkewY>),
}
impl TransformAnimate {
    fn parse_animate_transform(node: &Node) -> Result<Self, Error> {
        Ok(match get_attr(node, "type")? {
            "translate" => TransformAnimate::Translate(Animate::parse_animate_default(node)?),
            "scale" => TransformAnimate::Scale(Animate::parse_animate_default(node)?),
            "rotate" => TransformAnimate::Rotate(Animate::parse_animate_default(node)?),
            "skewX" => TransformAnimate::SkewX(Animate::parse_animate_default(node)?),
            "skewY" => TransformAnimate::SkewY(Animate::parse_animate_default(node)?),
            val => return Err(Error::InvalidAttributeValue(val.into())),
        })
    }
}

#[derive(Default, Clone, Debug)]
pub struct Transform {
    pub value: Transform2F,
    pub animations: Vec<TransformAnimate>
}
impl Transform {
    pub fn new(value: Transform2F) -> Transform {
        Transform { value, animations: Vec::new() }
    }
    pub fn parse_animate_node(&mut self, node: &Node) -> Result<(), Error> {
        self.animations.push(TransformAnimate::parse_animate_transform(node)?);
        Ok(())
    }
}
impl Parse for Transform {
    fn parse(s: &str) -> Result<Self, Error> {
        Ok(Transform::new(transform_list(s)?))
    }
}
#[derive(Debug, Clone)]
pub enum AnimationMode<T> {
    Absolute { from: T, to: T },
    Relative { by: T },
    Values { pairs: Vec<(f32, T)>, splines: Vec<UnitSpline> },
}
impl<T> AnimationMode<T> where T: Parse + Clone {
    pub fn parse_node(node: &Node, value: &T, calc_mode: CalcMode) -> Result<Self, Error> {
        let from = node.attribute("from");
        let to = node.attribute("to");

        if from.is_some() | to.is_some() {
            let from = from.map(T::parse).transpose()?.unwrap_or_else(|| value.clone());
            let to = to.map(T::parse).transpose()?.unwrap_or_else(|| value.clone());
            Ok(AnimationMode::Absolute { from, to })
        } else if let Some(by) = node.attribute("by") {
            let by = T::parse(by)?;
            Ok(AnimationMode::Relative { by })
        } else if let Some(values) = node.attribute("values") {
            let values = values.split(";").map(str::trim);
            let key_times = get_attr(node, "keyTimes")?.split(";").map(str::trim);
            
            let pairs = key_times.zip(values)
            .map(|(time, val)| {
                Ok((
                    f32::from_str(time)?,
                    T::parse(val)?
                ))
            })
            .collect::<Result<Vec<(f32, T)>, Error>>()?;
            
            let mut splines = vec![];
            if let CalcMode::Spline = calc_mode {
                splines = get_attr(node, "keySplines")?.split(";").map(|s| {
                    let [x1, y1, x2, y2] = number_list_4(s.trim())?;
                    Ok(UnitSpline(vec2f(x1, y1), vec2f(x2, y2)))
                }).collect::<Result<Vec<UnitSpline>, Error>>()?;
                if splines.len() + 1 != pairs.len() {
                    return Err(Error::InvalidAttributeValue("keySplines".into()));
                }
            }
            
            Ok(AnimationMode::Values { pairs, splines })
        } else {
            Err(Error::MissingAttribute("<animate> lacks from, to, by and values".into()))
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AnimationFill {
    Remove,
    Freeze
}
impl Parse for AnimationFill {
    fn parse(s: &str) -> Result<Self, Error> {
        match s {
            "freeze" => Ok(AnimationFill::Freeze),
            "remove" => Ok(AnimationFill::Remove),
            _ => Err(Error::InvalidAttributeValue(s.into()))
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Additive {
    Sum,
    Replace
}
impl Parse for Additive {
    fn parse(s: &str) -> Result<Self, Error> {
        match s {
            "sum" => Ok(Additive::Sum),
            "replace" => Ok(Additive::Replace),
            _ => Err(Error::InvalidAttributeValue(s.into()))
        }
    }
}
