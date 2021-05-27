use crate::prelude::*;
use crate::animate::*;

use pathfinder_content::{
    fill::{FillRule}
};
use svgtypes::{Length, Color};

wrap_option_iterpolate!(Fill);

wrap_option_iterpolate!(Stroke);

impl Resolve for Fill {
    type Output = Paint;
    fn resolve(&self, options: &Options) -> Self::Output {
        self.0.clone().unwrap_or_else(|| options.fill.clone())
    }
}
impl Resolve for Stroke {
    type Output = Paint;
    fn resolve(&self, options: &Options) -> Self::Output {
        self.0.clone().unwrap_or_else(|| options.stroke.clone())
    }
}