use std::ops::{Add, Sub, Mul};
use std::fmt::Debug;
use pathfinder_content::outline::Contour;
use crate::prelude::*;
use crate::parser::{number_list_4, one_or_two_numbers, one_or_three_numbers};

#[derive(Debug, Clone)]
struct Animate<T> {
    timing: Timing,
    mode: AnimationMode<T>,
    fill: AnimationFill,
    calc_mode: CalcMode,
    additive: Additive,
}
impl<T> Animate<T> where T: Parse + Clone {
    fn parse_animate(node: &Node, value: &T) -> Result<Self, Error> {
        let timing = Timing::parse_node(node)?;
        let calc_mode = parse_attr_or(node, "calcMode", CalcMode::Linear)?;
        let mode = AnimationMode::parse_node(node, value, calc_mode)?;
        let fill = parse_attr_or(node, "fill", AnimationFill::Remove)?;
        let default_additive = match mode {
            AnimationMode::Absolute { .. } | AnimationMode::Values { .. }=> Additive::Replace,
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
impl<T> Animate<T> where T: Resolve, T::Output: Interpolate {
    fn get_value(&self, options: &DrawOptions) -> Option<T::Output> {
        let x = self.timing.pos(options.time);
        if x < 0.0 {
            return None;
        }
        if x >= 1.0 {
            return match (self.fill, &self.mode) {
                (AnimationFill::Remove, _) => None,
                (AnimationFill::Freeze, AnimationMode::Absolute { to, .. }) => Some(to.resolve(options)),
                (AnimationFill::Freeze, AnimationMode::Relative { by }) => Some(by.resolve(options)),
                (AnimationFill::Freeze, AnimationMode::Values { ref pairs, .. }) => pairs.last().map(|(_, v)| v.resolve(options))
            };
        }

        match self.mode {
            AnimationMode::Absolute { ref from, ref to } => {
                Some(from.resolve(options).lerp(to.resolve(options), x))
            }
            AnimationMode::Relative { ref by } => {
                Some(by.resolve(options).scale(x))
            }
            AnimationMode::Values { ref pairs, ref splines } => {
                let val = |idx| pairs.get(idx).map(|&(t, ref v): &(f32, T)| v.resolve(options));
                let pos = pairs.binary_search_by(|&(y, _)| y.partial_cmp(&x).unwrap());
                match (self.calc_mode, pos) {
                    (CalcMode::Discrete, Ok(idx)) => val(idx),
                    (CalcMode::Discrete, Err(0)) => None,
                    (CalcMode::Discrete, Err(idx)) => val(idx - 1),
                    (CalcMode::Linear, Ok(idx)) => val(idx),
                    (mode, Err(idx)) if idx > 0 && idx < pairs.len() => {
                        let (t0, ref v0) = pairs[idx - 1];
                        let (t1, ref v1) = pairs[idx];
                        let fragment_time = (x - t0) / (t1 - t0);
                        let mapped_time = match mode {
                            CalcMode::Linear => fragment_time,
                            CalcMode::Spline => splines.get(idx - 1).unwrap().y_for_x(fragment_time),
                            _ => fragment_time // whatever
                        };
                        Some(v0.resolve(options).lerp(v1.resolve(options), mapped_time))
                    }
                    _ => None
                }
            }
        }
    }
    pub fn apply<U>(&self, base: U, options: &DrawOptions) -> U
        where T: Resolve, T::Output: Interpolate + Into<U>, U: Compose
    {
        match (self.additive, self.get_value(options)) {
            (Additive::Sum, Some(val)) => base.compose(val.into()),
            (Additive::Sum, None) => base,
            (Additive::Replace, Some(val)) => val.into(),
            (Additive::Replace, None) => base
        }
    }
}

#[derive(Debug, Clone)]
enum AnimationMode<T> {
    Absolute { from: T, to: T },
    Relative { by: T },
    Values { pairs: Vec<(f32, T)>, splines: Vec<UnitSpline> },
}
impl<T> AnimationMode<T> where T: Parse + Clone {
    fn parse_node(node: &Node, value: &T, calc_mode: CalcMode) -> Result<Self, Error> {
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
enum AnimationFill {
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
enum Additive {
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

struct AnimateMotion {
    path: Contour,
    path_len: f32,
    timing: Timing,
}

#[derive(Debug, Clone)]
struct Timing {
    begin: Time,
    scale: f32,
   //repeat_until: Time,
}
impl Timing {
    fn parse_node(node: &Node) -> Result<Timing, Error> {
        let begin = parse_attr_or(node, "begin", Time(0.0))?;
        let duration: Time = parse_attr(node, "dur")?;
        Ok(Timing { begin, scale: 1.0 / duration.seconds() })
    }
}

#[derive(Debug, Copy, Clone)]
enum CalcMode {
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
struct UnitSpline(Vector2F, Vector2F);
impl UnitSpline {
    fn at(&self, t: f32) -> Vector2F {
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
    fn y_for_x(&self, x: f32) -> f32 {
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
    fn pos(&self, t: Time) -> f32 {
        (t - self.begin).seconds() * self.scale
    }
}

pub trait Resolve {
    type Output;
    fn resolve(&self, options: &DrawOptions) -> Self::Output;
}
impl Resolve for LengthX {
    type Output = f32;
    fn resolve(&self, options: &DrawOptions) -> Self::Output {
        options.resolve_length_along(self.0, Axis::X).unwrap()
    }
}
impl Resolve for LengthY {
    type Output = f32;
    fn resolve(&self, options: &DrawOptions) -> Self::Output {
        options.resolve_length_along(self.0, Axis::Y).unwrap()
    }
}
impl Resolve for Vector {
    type Output = Vector2F;
    fn resolve(&self, options: &DrawOptions) -> Self::Output {
        options.resolve_vector(*self)
    }
}

resolve_clone!(f32);
resolve_clone!(Vector2F);

pub trait Interpolate: Clone {
    fn lerp(self, to: Self, x: f32) -> Self;
    fn scale(self, x: f32) -> Self;
}
pub trait Compose {
    fn compose(self, rhs: Self) -> Self;
}

impl Compose for Transform2F {
    fn compose(self, rhs: Self) -> Self {
        self * rhs
    }
}
impl Compose for f32 {
    fn compose(self, rhs: Self) -> Self {
        self + rhs
    }
}

primitive_interpolate!(f32);
primitive_interpolate!(Vector2F);

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

#[derive(Debug, Clone)]
pub struct Value<T> {
    value: T,
    animations: Vec<Animate<T>>,
}
impl<T> Value<T> {
    pub fn new(value: T) -> Value<T> {
        Value { value, animations: Vec::new() }
    }
}
impl<T> Value<T> where T: Resolve + Parse + Clone, T::Output: Interpolate + Compose {
    pub fn get(&self, options: &DrawOptions) -> T::Output {
        let base = self.value.resolve(options);
        self.animations.iter().fold(base, |base, animation| animation.apply(base, options))
    }
    pub fn parse_animate_node(&mut self, node: &Node) -> Result<(), Error> {
        self.animations.push(Animate::parse_animate(node, &self.value)?);
        Ok(())
    }
}
impl<T: Parse> Parse for Value<T> {
    fn parse(s: &str) -> Result<Self, Error> {
        T::parse(s).map(Value::new)
    }
}
impl<T: Parse + Default> Value<T> {
    pub fn parse_or_default(s: Option<&str>) -> Result<Self, Error> {
        Ok(Value::new(s.map(T::parse).transpose()?.unwrap_or_default()))
    }
}

#[derive(Clone, Debug, Default)]
struct Translation(Vector2F);
resolve_clone!(Translation);
wrap_interpolate!(Translation);
impl Into<Transform2F> for Translation {
    fn into(self) -> Transform2F {
        Transform2F::from_translation(self.0)
    }
}

#[derive(Clone, Debug)]
struct Scale(Vector2F);
resolve_clone!(Scale);
wrap_interpolate!(Scale);
impl Into<Transform2F> for Scale {
    fn into(self) -> Transform2F {
        Transform2F::from_scale(self.0)
    }
}

#[derive(Clone, Debug, Default)]
struct Rotation(f32, Vector2F);
resolve_clone!(Rotation);
impl Interpolate for Rotation {
    fn lerp(self, to: Self, x: f32) -> Self {
        Rotation(self.0.lerp(to.0, x), self.1.lerp(to.1, x))
    }
    fn scale(self, x: f32) -> Self {
        Rotation(self.0.scale(x), self.1.scale(x))
    }
}
impl Into<Transform2F> for Rotation {
    fn into(self) -> Transform2F {
        Transform2F::from_translation(self.1) * Transform2F::from_rotation(self.0) * Transform2F::from_translation(-self.1)
    }
}

#[derive(Clone, Debug, Default)]
struct SkewX(f32);
resolve_clone!(SkewX);
wrap_interpolate!(SkewX);
impl Into<Transform2F> for SkewX {
    fn into(self) -> Transform2F {
        skew_x(self.0)
    }
}

#[derive(Clone, Debug, Default)]
struct SkewY(f32);
resolve_clone!(SkewY);
wrap_interpolate!(SkewY);
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
enum TransformAnimate {
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
    animations: Vec<TransformAnimate>
}
impl Transform {
    pub fn get(&self, options: &DrawOptions) -> Transform2F {
        let base = self.value;
        self.animations.iter().fold(base, |base, animation| {
            let tr = match animation {
                TransformAnimate::Translate(ref anim) => anim.apply(base, options),
                TransformAnimate::Scale(ref anim) => anim.apply(base, options),
                TransformAnimate::Rotate(ref anim) => anim.apply(base, options),
                TransformAnimate::SkewX(ref anim) => anim.apply(base, options),
                TransformAnimate::SkewY(ref anim) => anim.apply(base, options)
            };
            dbg!(animation, tr);
            tr
        })
    }
    pub fn new(value: Transform2F) -> Transform {
        Transform { value, animations: Vec::new() }
    }
    pub fn parse_animate_transform(&mut self, node: &Node) -> Result<(), Error> {
        self.animations.push(TransformAnimate::parse_animate_transform(node)?);
        Ok(())
    }
}