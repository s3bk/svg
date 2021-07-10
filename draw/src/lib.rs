#[macro_use] extern crate log;

use pathfinder_renderer::{
    scene::{Scene}
};

#[macro_use]
mod macros;

mod prelude {
    pub use pathfinder_renderer::scene::Scene;
    pub use pathfinder_geometry::{
        vector::{Vector2F, vec2f},
        transform2d::Transform2F,
        rect::RectF,
    };
    pub use pathfinder_content::outline::Outline;
    pub use svg_dom::prelude::*;
    pub use crate::{
        DrawItem, Resolve, Interpolate, Compose, Shape,
        draw::{Options, DrawContext, BoundsOptions, DrawOptions},
    };
    pub use svgtypes::{Length, LengthUnit};
}
use std::borrow::Borrow;

mod path;
mod rect;
mod polygon;
mod ellipse;
mod attrs;
mod gradient;
mod resolve;
mod filter;
mod g;
mod draw;
mod svg;
#[cfg(feature="text")]
mod text;
mod animate;
mod paint;

pub use prelude::*;

#[cfg(feature="text")]
use svg_text::FontCollection;

use std::sync::Arc;

pub trait Resolve {
    type Output;
    fn resolve(&self, options: &Options) -> Self::Output;
    fn try_resolve(&self, options: &Options) -> Option<Self::Output> {
        Some(self.resolve(options))
    }
}

pub trait Shape {
    fn outline(&self, options: &Options) -> Option<Outline>;
}

pub trait DrawItem {
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions);
    fn bounds(&self, options: &BoundsOptions) -> Option<RectF>;
}

pub trait Interpolate: Clone {
    fn lerp(self, to: Self, x: f32) -> Self;
    fn scale(self, x: f32) -> Self;
}
impl<T> Interpolate for Option<T> where T: Interpolate {
    fn lerp(self, to: Self, x: f32) -> Self {
        match (self, to) {
            (Some(a), Some(b)) => Some(a.lerp(b, x)),
            _ => None
        }
    }
    fn scale(self, x: f32) -> Self {
        self.map(|v| v.scale(x))
    }
}

#[cfg(not(feature="text"))]
impl DrawItem for TagText {
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
    }
    fn bounds(&self, options: &BoundsOptions) -> Option<RectF> {
        None
    }
}

pub trait Compose {
    fn compose(self, rhs: Self) -> Self;
}
impl<T: Compose> Compose for Option<T> {
    fn compose(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Some(a), Some(b)) => Some(a.compose(b)),
            (a, b) => a.or(b)
        }
    }
}

macro_rules! draw_items {
    ($name:ident { $($variant:ident($data:ty), )* }) => {
        impl DrawItem for $name {
            fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
                match *self {
                    $( $name::$variant ( ref tag ) => tag.draw_to(scene, options), )*
                    _ => {}
                }
            }
            fn bounds(&self, options: &BoundsOptions) -> Option<RectF> {
                match *self {
                    $( $name::$variant ( ref tag ) => tag.bounds(options), )*
                    _ => None
                }
            }
        }
    }
}

draw_items!(
    Item {
        Path(TagPath),
        G(TagG),
        Rect(TagRect),
        Polygon(TagPolygon),
        Polyline(TagPolyline),
        Line(TagLine),
        Ellipse(TagEllipse),
        Circle(TagCircle),
        Svg(TagSvg),
        Use(TagUse),
        Text(TagText),
    }
);


use font::SvgGlyph;
pub fn draw_glyph(glyph: &SvgGlyph, scene: &mut Scene, transform: Transform2F) {
    let ctx = DrawContext::new_without_fonts(&*glyph.svg);
    let mut options = DrawOptions::new(&ctx);
    options.transform = transform * Transform2F::from_scale(Vector2F::new(1.0, -1.0));
    glyph.item.draw_to(scene, &options);
}
