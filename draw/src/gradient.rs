use crate::prelude::*;
use pathfinder_content::gradient::{Gradient};
use pathfinder_color::{ColorU};
use pathfinder_geometry::line_segment::LineSegment2F;
use pathfinder_simd::default::F32x2;
use svgtypes::Color;

struct PartialLinearGradient<'a> {
    from: (Option<LengthX>, Option<LengthY>),
    to: (Option<LengthX>, Option<LengthY>),
    gradient_transform: Option<Transform2F>,
    stops: &'a [TagStop],
}

struct PartialRadialGradient<'a> {
    center: (Option<LengthX>, Option<LengthY>),
    focus: (Option<LengthX>, Option<LengthY>),
    radius: Option<Length>,
    gradient_transform: Option<Transform2F>,
    stops: &'a [TagStop],
}

pub trait BuildGradient {
    fn build(&self, options: &Options, opacity: f32) -> Gradient;
}

impl BuildGradient for TagLinearGradient {
    fn build(&self, options: &Options, opacity: f32) -> Gradient {
        if let Some(item) = self.href.as_ref().and_then(|href| options.ctx.resolve_href(&href)) {
            match &**item {
                Item::LinearGradient(other) => {
                    return PartialLinearGradient {
                        from: merge_point(&self.from, &other.from),
                        to: merge_point(&self.to, &other.to),
                        gradient_transform: self.gradient_transform.or(other.gradient_transform),
                        stops: select_stops(&self.stops, &other.stops)
                    }.build(options, opacity)
                },
                Item::RadialGradient(other) => {
                    return PartialLinearGradient {
                        from: self.from,
                        to: self.to,
                        gradient_transform: self.gradient_transform,
                        stops: select_stops(&self.stops, &other.stops)
                    }.build(options, opacity)
                },
                _ => {}
            }
        }

        PartialLinearGradient {
            from: self.from,
            to: self.to,
            gradient_transform: self.gradient_transform,
            stops: &self.stops
        }.build(options, opacity)
    }
}

fn select_stops<'a>(a: &'a [TagStop], b: &'a [TagStop]) -> &'a [TagStop] {
    if a.len() > 0 {
        a
    } else {
        b
    }
}

fn merge_point(
    a: &(Option<LengthX>, Option<LengthY>),
    b: &(Option<LengthX>, Option<LengthY>)
) -> (Option<LengthX>, Option<LengthY>) {
    (
        a.0.or(b.0),
        a.1.or(b.1)
    )
}
fn length_or_percent(a: Option<Length>, default: f64) -> Length {
    match a {
        Some(l) => l,
        None => Length::new(default, LengthUnit::Percent)
    }
}
fn point_or_percent((x, y): (Option<LengthX>, Option<LengthY>), (dx, dy): (f64, f64)) -> Vector {
    Vector(
        x.unwrap_or(LengthX(Length::new(dx, LengthUnit::Percent))),
        y.unwrap_or(LengthY(Length::new(dy, LengthUnit::Percent))),
    )
}

impl BuildGradient for TagRadialGradient {
    fn build(&self, options: &Options, opacity: f32) -> Gradient {
        if let Some(item) = self.href.as_ref().and_then(|href| options.ctx.resolve(&href[1..])) {
            match &**item {
                Item::RadialGradient(ref other) => {
                    return PartialRadialGradient {
                        center: merge_point(&self.center, &other.center),
                        focus: merge_point(&self.focus, &other.focus),
                        radius: self.radius.or(other.radius),
                        gradient_transform: self.gradient_transform.or(other.gradient_transform),
                        stops: select_stops(&self.stops, &other.stops)
                    }.build(options, opacity)
                }
                Item::LinearGradient(ref other) => {
                    return PartialRadialGradient {
                        center: self.center,
                        focus: self.focus,
                        radius: self.radius,
                        gradient_transform: self.gradient_transform,
                        stops: select_stops(&self.stops, &other.stops)
                    }.build(options, opacity)
                }
                _ => {}
            }
        }
        PartialRadialGradient {
            center: self.center,
            focus: self.focus,
            radius: self.radius,
            gradient_transform: self.gradient_transform,
            stops: &self.stops
        }.build(options, opacity)
    }
}

impl<'a> PartialLinearGradient<'a> {
    fn build(self, options: &Options, opacity: f32) -> Gradient {
        let from = point_or_percent(self.from, (0., 0.));
        let to = point_or_percent(self.to, (100., 0.));
        let gradient_transform = self.gradient_transform.unwrap_or_default();

        let mut gradient = Gradient::linear_from_points(
            from.resolve(options),
            to.resolve(options),
        );
        for stop in self.stops {
            gradient.add_color_stop(stop.color_u(opacity), stop.offset);
        }

        gradient.apply_transform(options.transform * gradient_transform);
        gradient
    }
}
impl<'a> PartialRadialGradient<'a> {
    fn build(&self, options: &Options, opacity: f32) -> Gradient {
        let center = point_or_percent(self.center, (50., 50.));
        let focus = Vector(self.focus.0.unwrap_or(center.0), self.focus.1.unwrap_or(center.1));
        let radius = length_or_percent(self.radius, 50.);
        let gradient_transform = self.gradient_transform.unwrap_or_default();

        let mut gradient = Gradient::radial(
            LineSegment2F::new(
                focus.resolve(options),
                center.resolve(options)
            ),
            F32x2::new(0.0, options.resolve_length(radius).unwrap())
        );
        for stop in self.stops {
            gradient.add_color_stop(stop.color_u(opacity), stop.offset);
        }

        gradient.apply_transform(options.transform * gradient_transform);
        gradient
    }
}