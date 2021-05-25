use crate::prelude::*;
use pathfinder_content::outline::{Outline, Contour};

pub fn rect_outline<'a>(rect: &TagRect, options: &DrawOptions<'a>) -> Option<(DrawOptions<'a>, Outline)> {
    if !rect.attrs.display {
        return None;
    }
    let options = options.apply(&rect.attrs);

    let size = rect.size.resolve(&options);
    if (size.x() == 0.) || (size.y() == 0.) {
        return None;
    }
    
    let origin = rect.pos.resolve(&options);
    let radius = rect.radius.resolve(&options);
    let contour = Contour::from_rect_rounded(RectF::new(origin, size), radius);

    let mut outline = Outline::with_capacity(1);
    outline.push_contour(contour);
    Some((options, outline))
}

impl DrawItem for TagRect {
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        if let Some((options, outline)) = rect_outline(self, &options) {
            options.draw(scene, &outline);
        }
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

        options.bounds(RectF::new(origin, size))
    }
}
