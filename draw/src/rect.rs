use crate::prelude::*;
use pathfinder_content::outline::{Outline, Contour};

fn rect_outline<'a>(tag: &TagRect, options: &Options<'a>) -> Option<Outline> {
    if !tag.attrs.display {
        return None;
    }

    let size = tag.size.resolve(&options);
    if (size.x() == 0.) || (size.y() == 0.) {
        return None;
    }
    
    let origin = tag.pos.resolve(&options);
    let rx = tag.rx.resolve(&options);
    let ry = tag.ry.resolve(&options);
    let rect = RectF::new(origin, size);

    let contour = match (rx, ry) {
        (Some(x), Some(y)) => Contour::from_rect_rounded(rect, Vector2F::new(x, y)),
        (Some(r), None) | (None, Some(r)) => Contour::from_rect_rounded(rect, Vector2F::new(r, r)),
        (None, None) => Contour::from_rect(rect),
    };

    let mut outline = Outline::with_capacity(1);
    outline.push_contour(contour);
    Some(outline)
}

impl Shape for TagRect {
    fn outline(&self, options: &Options) -> Option<Outline> {
        let options = options.apply(&self.attrs);
        rect_outline(self, &options).map(|o| o.transformed(options.get_transform()))
    }
}
impl DrawItem for TagRect {
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(scene, &self.attrs);
        if let Some(outline) = rect_outline(self, &options) {
            options.draw(scene, &outline);
        }
    }
    fn bounds(&self, options: &BoundsOptions) -> Option<RectF> {
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
