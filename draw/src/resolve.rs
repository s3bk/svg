use crate::prelude::*;


fn apply_anim<T, U>(animate: &Animate<T>, base: U, options: &DrawOptions) -> U
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
    fn resolve(&self, options: &DrawOptions) -> T::Output {
        let base = self.value.resolve(options);
        self.animations.iter().fold(base, |base, animation| apply_anim(animation, base, options))
    }
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

impl Resolve for ValueVector {
    type Output = Vector2F;
    fn resolve(&self, options: &DrawOptions) -> Vector2F {
        let x = self.x.resolve(options);
        let y = self.y.resolve(options);
        vec2f(x, y)
    }
}

impl Resolve for Transform {
    type Output = Transform2F;
    fn resolve(&self, options: &DrawOptions) -> Transform2F {
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