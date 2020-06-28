use crate::prelude::*;
use palette::{
    rgb::{LinSrgb, Srgb},
};
use pathfinder_color::{ColorF, ColorU};

impl Interpolate for Color {
    fn lerp(self, to: Self, x: f32) -> Self {
        Color {
            red: self.red.lerp(to.red, x),
            green: self.green.lerp(to.green, x),
            blue: self.blue.lerp(to.blue, x),
        }
    }
    fn scale(self, x: f32) -> Self {
        Color {
            red: self.red.scale(x),
            green: self.green.scale(x),
            blue: self.blue.scale(x),
        }
    }
}
impl Compose for Color {
    fn compose(self, rhs: Self) -> Self {
        Color {
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            blue: self.blue + rhs.blue,
        }
    }
}

impl Interpolate for Paint {
    fn lerp(self, to: Self, x: f32) -> Self {
        match (self, to) {
            (Paint::Color(a), Paint::Color(b)) => Paint::Color(a.lerp(b, x)),
            (Paint::None, b) => b,
            (a, _) => a
        }
    }
    fn scale(self, x: f32) -> Self {
        match self {
            Paint::Color(a) => Paint::Color(a.scale(x)),
            p => p
        }
    }
}
impl Compose for Paint {
    fn compose(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Paint::Color(a), Paint::Color(b)) => Paint::Color(a.compose(b)),
            (Paint::None, b) => b,
            (a, _) => a
        }
    }
}
