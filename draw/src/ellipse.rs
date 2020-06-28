use crate::prelude::*;
use pathfinder_content::outline::{Outline, Contour};

impl DrawItem for TagEllipse {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if !self.radius.has_area() || !self.attrs.display {
            return None;
        }
        let options = options.apply(&self.attrs);
        let center = options.resolve_vector(self.center);
        let radius = options.resolve_vector(self.radius);

        options.bounds(RectF::new(center - radius, radius * 2.0))
    }

    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        if !self.radius.has_area() || !self.attrs.display {
            return;
        }
        let options = options.apply(&self.attrs);

        let center = options.resolve_vector(self.center);
        let radius = options.resolve_vector(self.radius);

        let mut contour = Contour::with_capacity(4);
        let tr = Transform2F::from_translation(center) * Transform2F::from_scale(radius);
        contour.push_ellipse(&tr);

        let mut outline = Outline::with_capacity(1);
        outline.push_contour(contour);

        options.draw(scene, &outline);
    }
}