use std::ops::{Add, Sub, Mul};
use std::fmt::Debug;
use pathfinder_content::outline::Contour;
use crate::prelude::*;
use std::rc::Rc;

impl<T> Resolve for Animate<T> where T: Resolve, T::Output: Interpolate {
    type Output = Option<T::Output>;
    fn resolve(&self, options: &Options) -> Option<T::Output> {
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

resolve_clone!(Translation);
wrap_interpolate!(Translation);
resolve_clone!(Scale);
wrap_interpolate!(Scale);

resolve_clone!(Rotation);
impl Interpolate for Rotation {
    fn lerp(self, to: Self, x: f32) -> Self {
        Rotation(self.0.lerp(to.0, x), self.1.lerp(to.1, x))
    }
    fn scale(self, x: f32) -> Self {
        Rotation(self.0.scale(x), self.1.scale(x))
    }
}

wrap_interpolate!(SkewX);
wrap_interpolate!(SkewY);

impl Interpolate for Rc<[f32]> {
    fn lerp(self, to: Self, x: f32) -> Self {
        let mut out = Vec::with_capacity(self.len());
        out.extend(self.iter().zip(to.iter()).map(|(&from, &to)| from.lerp(to, x)));
        out.into()
    }
    fn scale(self, x: f32) -> Self {
        let mut out = Vec::with_capacity(self.len());
        out.extend(self.iter().map(|&val| val.scale(x)));
        out.into()
    }
}
impl Compose for Rc<[f32]> {
    fn compose(self, rhs: Self) -> Self {
        rhs
    }
}
