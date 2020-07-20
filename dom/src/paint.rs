use crate::prelude::*;
use crate::parser::{parse_color, parse_paint};
use pathfinder_color::{ColorF, ColorU};

#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32
}
impl Color {
    pub fn from_srgb_u8(r: u8, g: u8, b: u8) -> Color {
        Color {
            red: r as f32 * (1.0/255.),
            green: g as f32 * (1.0/255.),
            blue: b as f32 * (1.0/255.)
        }
    }
    pub fn black() -> Color {
        Color {
            red: 0.,
            green: 0.,
            blue: 0.
        }
    }
    pub fn color_f(&self, alpha: f32) -> ColorF {
        ColorF::new(self.red, self.green, self.blue, alpha)
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
#[test]
fn test_color() {
    assert_eq!(Color::parse("#aabbcc").unwrap(), Color::from_srgb_u8(0xaa, 0xbb, 0xcc));
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
#[test]
fn test_paint() {
    assert_eq!(Paint::parse("#aabbcc").unwrap(), Paint::Color(Color::from_srgb_u8(0xaa, 0xbb, 0xcc)));
}