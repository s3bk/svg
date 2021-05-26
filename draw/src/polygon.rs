use crate::prelude::*;

use pathfinder_content::outline::{Outline, Contour};
use svgtypes::PointsParser;

impl DrawItem for TagPolygon {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if self.attrs.display && self.outline.len() > 0 {
            let options = options.apply(&self.attrs);
            options.bounds(self.outline.bounds())
        } else {
            None
        }
    }
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(&self.attrs);
        options.draw(scene, &self.outline);
    }
}

impl DrawItem for TagPolyline {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if self.attrs.display && self.outline.len() > 0 {
            let options = options.apply(&self.attrs);
            options.bounds(self.outline.bounds())
        } else {
            None
        }
    }
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(&self.attrs);
        options.draw(scene, &self.outline);
    }
}
