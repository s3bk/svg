use std::ops::{Add, Sub};
use pathfinder_content::outline::Contour;
use crate::prelude::*;
use crate::parser::number_list_4;

#[derive(Debug, Clone)]
struct Animate<T> {
    timing: Timing,
    mode: AnimationMode<T>,
    fill: AnimationFill,
    splines: Vec<UnitSpline>,
    calc_mode: CalcMode,
}
impl<T> Animate<T> where T: Resolve, T::Output: Interpolate {
    pub fn apply(&self, base: T::Output, options: &DrawOptions) -> T::Output {
        let x = self.timing.pos(options.time);
        if x < 0.0 {
            return base;
        }
        if x >= 1.0 {
            return match (self.fill, &self.mode) {
                (AnimationFill::Remove, _) => base,
                (AnimationFill::Freeze, AnimationMode::Absolute { to, .. }) => to.resolve(options),
                (AnimationFill::Freeze, AnimationMode::Relative { by }) => Interpolate::add(base, by.resolve(options)),
                (AnimationFill::Freeze, AnimationMode::Values( ref pairs )) => pairs.last().map(|&(_, ref v)| v.resolve(options)).unwrap_or(base)
            };
        }
        match self.mode {
            AnimationMode::Absolute { ref from, ref to } => {
                Interpolate::linear(from.resolve(options), to.resolve(options), x)
            }
            AnimationMode::Relative { ref by } => {
                Interpolate::add_interpolate(base, by.resolve(options), x)
            }
            AnimationMode::Values( ref pairs ) => {
                let val = |idx| pairs.get(idx).map(|&(t, ref v): &(f32, T)| v.resolve(options));
                let pos = pairs.binary_search_by(|&(y, _)| y.partial_cmp(&x).unwrap());
                match (self.calc_mode, pos) {
                    (CalcMode::Discrete, Ok(idx)) => val(idx).unwrap_or(base),
                    (CalcMode::Discrete, Err(0)) => base,
                    (CalcMode::Discrete, Err(idx)) => val(idx - 1).unwrap_or(base),
                    (CalcMode::Linear, Ok(idx)) => val(idx).unwrap_or(base),
                    (mode, Err(idx)) if idx > 0 && idx < pairs.len() => {
                        let (t0, ref v0) = pairs[idx - 1];
                        let (t1, ref v1) = pairs[idx];
                        let fragment_time = (x - t0) / (t1 - t0);
                        let mapped_time = match mode {
                            CalcMode::Linear => fragment_time,
                            CalcMode::Spline => self.splines.get(idx - 1).unwrap().y_for_x(fragment_time),
                            _ => fragment_time // whatever
                        };
                        Interpolate::linear(v0.resolve(options), v1.resolve(options), mapped_time)
                    }
                    _ => base
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum AnimationMode<T> {
    Absolute { from: T, to: T },
    Relative { by: T },
    Values(Vec<(f32, T)>),
}

#[derive(Debug, Copy, Clone)]
enum AnimationFill {
    Remove,
    Freeze
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
            dbg!(x, low, mid, high, p);
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
        dbg!(delta, f_high, f_low, p);

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
impl Resolve for Length {
    type Output = f32;

    fn resolve(&self, options: &DrawOptions) -> Self::Output {
        options.resolve_length(*self)
    }
}

pub trait Interpolate: Clone {
    fn linear(from: Self, to: Self, x: f32) -> Self;
    fn add(a: Self, b: Self) -> Self;
    fn add_interpolate(base: Self, by: Self, x: f32) -> Self {
        Interpolate::linear(base.clone(), Interpolate::add(base, by), x)
    }
}
impl<T: Interpolate> Interpolate for Option<T> {
    fn linear(from: Self, to: Self, x: f32) -> Self {
        match (from, to) {
            (Some(a), Some(b)) => Some(Interpolate::linear(a, b, x)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None
        }
    }
    fn add(a: Self, b: Self) -> Self {
        match (a, b) {
            (Some(a), Some(b)) => Some(Interpolate::add(a, b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None
        }
    }
}
impl Interpolate for f32 {
    fn linear(from: Self, to: Self, x: f32) -> Self {
        (1.0 - x) * from + x * to
    }
    fn add(a: Self, b: Self) -> Self {
        a + b
    }
    fn add_interpolate(base: Self, by: Self, x: f32) -> Self {
        base + x * by
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

#[derive(Debug, Clone)]
pub struct Value<T> {
    base: T,
    animations: Vec<Animate<T>>,
}
impl<T> Value<T> {
    pub fn new(base: T) -> Value<T> {
        Value { base, animations: Vec::new() }
    }
}
impl<T> Value<T> where T: Resolve + Parse + Clone, T::Output: Interpolate {
    pub fn get(&self, options: &DrawOptions) -> T::Output {
        let base = self.base.resolve(options);
        self.animations.iter().fold(base, |base, animation| animation.apply(base, options))
    }
    pub fn parse_animate_node(&mut self, node: &Node) -> Result<(), Error> {
        let begin = Time::parse(node.attribute("begin").unwrap())?;
        let duration = Time::parse(node.attribute("dur").unwrap())?;
        let timing = Timing { begin, scale: 1.0 / duration.seconds() };

        let from = node.attribute("from");
        let to = node.attribute("to");
        
        let calc_mode = parse_attr_or(node, "calcMode", CalcMode::Linear)?;
        let mut splines = vec![];
        
        let mode = if from.is_some() | to.is_some() {
            let from = from.map(T::parse).transpose()?.unwrap_or_else(|| self.base.clone());
            let to = to.map(T::parse).transpose()?.unwrap_or_else(|| self.base.clone());
            AnimationMode::Absolute { from, to }
        } else if let Some(by) = node.attribute("by") {
            let by = T::parse(by)?;
            AnimationMode::Relative { by }
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
            
            if let CalcMode::Spline = calc_mode {
                splines = get_attr(node, "keySplines")?.split(";").map(|s| {
                    let [x1, y1, x2, y2] = number_list_4(s.trim())?;
                    Ok(UnitSpline(vec2f(x1, y1), vec2f(x2, y2)))
                }).collect::<Result<Vec<UnitSpline>, Error>>()?;
                if splines.len() + 1 != pairs.len() {
                    return Err(Error::InvalidAttributeValue("keySplines".into()));
                }
            };
            
            AnimationMode::Values(pairs)
        } else {
            warn!("<animate> lacks from, to, by and values");
            return Ok(());
        };

        let fill = match node.attribute("fill") {
            Some("freeze") => AnimationFill::Freeze,
            Some("remove") | None => AnimationFill::Remove,
            Some(val) => return Err(Error::InvalidAttributeValue(val.into()))
        };

        let animate = Animate {
            timing,
            mode,
            fill,
            calc_mode,
            splines,
        };
        self.animations.push(animate);
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
