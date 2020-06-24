use crate::prelude::*;
use crate::parser::{parse_color, parse_paint};
use palette::{
    rgb::{LinSrgb, Srgb},
};
use pathfinder_color::{ColorF, ColorU};

#[derive(Debug, Clone, PartialEq)]
pub struct Color(LinSrgb<f32>);
impl Color {
    pub fn from_srgb_u8(r: u8, g: u8, b: u8) -> Color {
        let srgb: Srgb<f32> = Srgb::from_components((r, g, b)).into_format();
        Color(srgb.into_linear())
    }
    pub fn black() -> Color {
        Color(LinSrgb::new(0., 0., 0.))
    }
    pub fn color_f(&self, alpha: f32) -> ColorF {
        let (red, green, blue) = Srgb::from_linear(self.0).into_components();
        ColorF::new(red, green, blue, alpha)
    }
    pub fn color_u(&self, alpha: f32) -> ColorU {
        self.color_f(alpha).to_u8()
    }
}
impl Parse for Color {
    fn parse(s: &str) -> Result<Self, Error> {
        parse_color(s)
    }
}
impl Interpolate for Color {
    fn linear(from: Self, to: Self, x: f32) -> Self {
        Color(to.0 * x + from.0 * (1.0 - x))
    }
    fn add(a: Self, b: Self) -> Self {
        Color(a.0 + b.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Paint {
    None,
    Color(Color),
    Ref(String),
}
impl Paint {
    pub fn is_none(&self) -> bool {
        matches!(*self, Paint::None)
    }
    pub fn black() -> Paint {
        Paint::Color(Color::black())
    }
    pub fn is_visible(&self) -> bool {
        match *self {
            Paint::None => false,
            _ => true,
        }
    }
}
impl Parse for Paint {
    fn parse(s: &str) -> Result<Self, Error> {
        parse_paint(s)
    }
}
impl Interpolate for Paint {
    fn linear(from: Self, to: Self, x: f32) -> Self {
        match (from, to) {
            (Paint::Color(a), Paint::Color(b)) => Paint::Color(Interpolate::linear(a, b, x)),
            (Paint::None, b) => b,
            (a, _) => a
        }
    }
    fn add(a: Self, b: Self) -> Self {
        match (a, b) {
            (Paint::Color(a), Paint::Color(b)) => Paint::Color(Interpolate::add(a, b)),
            (Paint::None, b) => b,
            (a, _) => a
        }
    }
}
