use crate::prelude::*;
use pathfinder_content::outline::{Outline, Contour};

impl DrawItem for TagRect {
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        if !self.attrs.display {
            return;
        }
        let options = options.apply(&self.attrs);

        let size = self.size.resolve(&options);
        if (size.x() == 0.) || (size.y() == 0.) {
            return;
        }
        
        let origin = self.pos.resolve(&options);
        let radius = self.radius.resolve(&options);
        let contour = Contour::from_rect_rounded(RectF::new(origin, size), radius);

        let mut outline = Outline::with_capacity(1);
        outline.push_contour(contour);

        options.draw(scene, &outline);
    }
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if !self.attrs.display {
            return None;
        }
        let options = options.apply(&self.attrs);

        let size = self.size.resolve(&options);
        if (size.x() == 0.) || (size.y() == 0.) {
            return None;
        }
        
        let origin = self.pos.resolve(&options);

        Some(RectF::new(origin, size))
    }
}
