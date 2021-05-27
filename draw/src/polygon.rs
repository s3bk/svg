use crate::prelude::*;

use pathfinder_content::outline::{Outline, Contour};
use svgtypes::PointsParser;

impl Shape for TagPolygon {
    fn outline(&self, options: &Options) -> Option<Outline> {
        let options = options.apply(&self.attrs);
        Some(self.outline.clone().transformed(options.get_transform()))
    }
}
impl DrawItem for TagPolygon {
    fn bounds(&self, options: &BoundsOptions) -> Option<RectF> {
        if self.attrs.display && self.outline.len() > 0 {
            let options = options.apply(&self.attrs);
            options.bounds(self.outline.bounds())
        } else {
            None
        }
    }
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(scene, &self.attrs);
        options.draw(scene, &self.outline);
    }
}

impl DrawItem for TagPolyline {
    fn bounds(&self, options: &BoundsOptions) -> Option<RectF> {
        if self.attrs.display && self.outline.len() > 0 {
            let options = options.apply(&self.attrs);
            options.bounds(self.outline.bounds())
        } else {
            None
        }
    }
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(scene, &self.attrs);
        options.draw(scene, &self.outline);
    }
}

impl DrawItem for TagLine {
    fn bounds(&self, options: &BoundsOptions) -> Option<RectF> {
        if self.attrs.display {
            let options = options.apply(&self.attrs);
            let p1 = self.p1.resolve(&options);
            let p2 = self.p2.resolve(&options);
            Some(RectF::from_points(p1.min(p2), p1.max(p2)))
        } else {
            None
        }
    }
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(scene, &self.attrs);
        let p1 = self.p1.resolve(&options);
        let p2 = self.p2.resolve(&options);

        let mut contour = Contour::with_capacity(2);
        contour.push_endpoint(p1);
        contour.push_endpoint(p2);

        let mut outline = Outline::with_capacity(1);
        outline.push_contour(contour);

        options.draw(scene, &outline);
    }
}

