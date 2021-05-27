use crate::prelude::*;
use std::rc::Rc;

fn apply_anim<T, U>(animate: &Animate<T>, base: U, options: &Options) -> U
where T: Resolve, T::Output: Interpolate + Into<U>, U: Compose
{
    match (animate.additive, animate.resolve(options)) {
        (Additive::Sum, Some(val)) => base.compose(val.into()),
        (Additive::Replace, Some(val)) => val.into(),
        (_, None) => base,
    }
}
impl<T> Resolve for Value<T> where T: Resolve + Parse + Clone, T::Output: Interpolate + Compose {
    type Output = T::Output;
    fn resolve(&self, options: &Options) -> T::Output {
        let base = self.value.resolve(options);
        self.animations.iter().fold(base, |base, animation| apply_anim(animation, base, options))
    }
}
impl<T> Resolve for Option<T> where T: Resolve {
    type Output = Option<T::Output>;
    fn resolve(&self, options: &Options) -> Option<T::Output> {
        self.as_ref().map(|val| val.resolve(options))
    }
    fn try_resolve(&self, options: &Options) -> Option<Self::Output> {
        self.as_ref().map(|val| val.try_resolve(options))
    }
}

impl Resolve for Length {
    type Output = f32;
    fn resolve(&self, options: &Options) -> Self::Output {
        options.resolve_length(*self).unwrap()
    }
    fn try_resolve(&self, options: &Options) -> Option<Self::Output> {
        options.resolve_length(*self)
    }
}

impl Resolve for LengthX {
    type Output = f32;
    fn resolve(&self, options: &Options) -> Self::Output {
        options.resolve_length_along(self.0, Axis::X).unwrap()
    }
    fn try_resolve(&self, options: &Options) -> Option<Self::Output> {
        options.resolve_length_along(self.0, Axis::X)
    }
}
impl Resolve for LengthY {
    type Output = f32;
    fn resolve(&self, options: &Options) -> Self::Output {
        options.resolve_length_along(self.0, Axis::Y).unwrap()
    }
    fn try_resolve(&self, options: &Options) -> Option<Self::Output> {
        options.resolve_length_along(self.0, Axis::Y)
    }
}
impl Resolve for Vector {
    type Output = Vector2F;
    fn resolve(&self, options: &Options) -> Self::Output {
        vec2f(self.0.resolve(options), self.1.resolve(options))
    }
    fn try_resolve(&self, options: &Options) -> Option<Self::Output> {
        match (self.0.try_resolve(options), self.1.try_resolve(options)) {
            (Some(x), Some(y)) => Some(vec2f(x, y)),
            _ => None
        }
    }
}
impl Resolve for Rect {
    type Output = RectF;
    fn resolve(&self, options: &Options) -> Self::Output {
        RectF::new(self.origin().resolve(options), self.size().resolve(options))
    }
    fn try_resolve(&self, options: &Options) -> Option<Self::Output> {
        match (self.origin().try_resolve(options), self.size().try_resolve(options)) {
            (Some(origin), Some(size)) => Some(RectF::new(origin, size)),
            _ => None
        }
    }
}
impl Resolve for ValueVector {
    type Output = Vector2F;
    fn resolve(&self, options: &Options) -> Vector2F {
        let x = self.x.resolve(options);
        let y = self.y.resolve(options);
        vec2f(x, y)
    }
}

impl Resolve for DashArray {
    type Output = Rc<[f32]>;
    fn resolve(&self, options: &Options) -> Rc<[f32]> {
        let mut out = Vec::with_capacity(self.0.len());
        for len in self.0.iter() {
            out.push(len.resolve(options));
        }
        out.into()
    }
    fn try_resolve(&self, options: &Options) -> Option<Rc<[f32]>> {
        let mut out = Vec::with_capacity(self.0.len());
        for len in self.0.iter() {
            out.push(len.try_resolve(options)?);
        }
        Some(out.into())
    }
}

impl Resolve for Transform {
    type Output = Transform2F;
    fn resolve(&self, options: &Options) -> Transform2F {
        let base = self.value;
        self.animations.iter().fold(base, |base, animation| match animation {
            TransformAnimate::Translate(ref anim) => apply_anim(anim, base, options),
            TransformAnimate::Scale(ref anim) => apply_anim(anim, base, options),
            TransformAnimate::Rotate(ref anim) => apply_anim(anim, base, options),
            TransformAnimate::SkewX(ref anim) => apply_anim(anim, base, options),
            TransformAnimate::SkewY(ref anim) => apply_anim(anim, base, options),
        })
    }
}

resolve_clone!(f32);
resolve_clone!(Vector2F);

resolve_clone!(SkewX);
resolve_clone!(SkewY);