use std::ops::{Add, Sub};
use pathfinder_content::outline::Contour;
use crate::prelude::*;

#[derive(Debug, Clone)]
struct Animate<T> {
    timing: Timing,
    mode: AnimationMode<T>,
    fill: AnimationFill,
}
impl<T> Animate<T> where T: Resolve, T::Output: Interpolate {
    pub fn apply(&self, base: T::Output, options: &DrawOptions) -> T::Output {
        let x = self.timing.pos(options.time);
        if x < 0.0 {
            return base;
        }
        if x > 1.0 {
            return match (self.fill, &self.mode) {
                (AnimationFill::Remove, _) => base,
                (AnimationFill::Freeze, AnimationMode::Absolute { to, .. }) => to.resolve(options),
                (AnimationFill::Freeze, AnimationMode::Relative { by }) => Interpolate::add(base, by.resolve(options)),
            };
        }
        match self.mode {
            AnimationMode::Absolute { ref from, ref to } => {
                Interpolate::linear(from.resolve(options), to.resolve(options), x)
            }
            AnimationMode::Relative { ref by } => {
                Interpolate::add_interpolate(base, by.resolve(options), x)
            }
        }
    }
}

#[derive(Debug, Clone)]
enum AnimationMode<T> {
    Absolute { from: T, to: T },
    Relative { by: T },
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
}
impl Timing {
    pub fn from_begin_and_duration(begin: Time, duration: Time) -> Timing {
        let scale = 1.0 / duration.seconds();
        Timing { begin, scale }
    }
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
        let timing = Timing::from_begin_and_duration(begin, duration);

        let from = node.attribute("from");
        let to = node.attribute("to");
        
        let mode = if from.is_some() | to.is_some() {
            let from = from.map(T::parse).transpose()?.unwrap_or_else(|| self.base.clone());
            let to = to.map(T::parse).transpose()?.unwrap_or_else(|| self.base.clone());
            AnimationMode::Absolute { from, to }
        } else if let Some(by) = node.attribute("by") {
            let by = T::parse(by)?;
            AnimationMode::Relative { by }
        } else {
            warn!("<animate> lacks from, to and by");
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
            fill
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
