#[cfg(feature="text")]
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

use pathfinder_renderer::{
    scene::{Scene}
};
use std::sync::Arc;

#[macro_use]
mod macros;

mod prelude {
    pub use pathfinder_renderer::scene::Scene;
    pub use pathfinder_geometry::{
        vector::{Vector2F, vec2f},
        transform2d::Transform2F,
        rect::RectF,
    };
    pub use svg_dom::prelude::*;
    pub use crate::{
        DrawItem, Resolve, Interpolate, Compose,
        draw::{DrawOptions, DrawContext},
    };
    pub use svgtypes::{Length, LengthUnit};
}

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
mod animate;
mod paint;

#[cfg(feature="text")]
mod text;

pub use prelude::*;

pub trait Resolve {
    type Output;
    fn resolve(&self, options: &DrawOptions) -> Self::Output;
}

pub trait DrawItem {
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions);
    fn bounds(&self, options: &DrawOptions) -> Option<RectF>;
}

pub trait Interpolate: Clone {
    fn lerp(self, to: Self, x: f32) -> Self;
    fn scale(self, x: f32) -> Self;
}

pub trait Compose {
    fn compose(self, rhs: Self) -> Self;
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
            fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
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
        Ellipse(TagEllipse),
        Svg(TagSvg),
        Use(TagUse),
    }
);

pub struct DrawSvg {
    svg: Svg
}
impl DrawSvg {
    pub fn new(svg: Svg) -> DrawSvg {
        DrawSvg { svg }
    }
    pub fn compose(&self) -> Scene {
        self.compose_with_transform(Transform2F::default())
    }

    pub fn compose_with_transform(&self, transform: Transform2F) -> Scene {
        let ctx = self.ctx();
        let mut options = DrawOptions::new(&ctx);
        options.transform = transform;
        self.compose_with_options(&options)
    }

    pub fn compose_with_options(&self, options: &DrawOptions) -> Scene {
        let mut scene = Scene::new();
        
        if let Item::Svg(TagSvg { view_box: Some(r), .. }) = &*self.svg.root {
            scene.set_view_box(options.transform * options.resolve_rect(r));
        }
        self.svg.root.draw_to(&mut scene, options);
        scene
    }

    /// get the viewbox (computed if missing)
    pub fn view_box(&self) -> Option<RectF> {
        let ctx = self.ctx();
        let options = DrawOptions::new(&ctx);
        
        if let Item::Svg(TagSvg { view_box: Some(r), .. }) = &*self.svg.root {
            return Some(options.resolve_rect(r));
        } else {
            self.svg.root.bounds(&options)
        }
    }

    pub fn ctx(&self) -> DrawContext {
        DrawContext::new(&self.svg)
    }
}
