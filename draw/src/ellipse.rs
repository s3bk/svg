use crate::prelude::*;
use pathfinder_content::outline::{Outline, Contour};

impl DrawItem for TagEllipse {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if !self.attrs.display {
            return None;
        }
        let options = options.apply(&self.attrs);
        let center = self.center.resolve(&options);
        let radius = self.radius.resolve(&options);

        if radius.x() == 0.0 || radius.y() == 0.0 {
            return None;
        }

        options.bounds(RectF::new(center - radius, radius * 2.0))
    }

    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        if !self.attrs.display {
            return;
        }
        let options = options.apply(&self.attrs);

        let center = self.center.resolve(&options);
        let radius = self.radius.resolve(&options);

        if radius.x() == 0.0 || radius.y() == 0.0 {
            return;
        }

        let mut contour = Contour::with_capacity(4);
        let tr = Transform2F::from_translation(center) * Transform2F::from_scale(radius);
        contour.push_ellipse(&tr);

        let mut outline = Outline::with_capacity(1);
        outline.push_contour(contour);

        options.draw(scene, &outline);
    }
}

impl DrawItem for TagCircle {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if !self.attrs.display {
            return None;
        }
        let options = options.apply(&self.attrs);
        let center = self.center.resolve(&options);
        let radius = self.radius.resolve(&options);

        if radius == 0.0 {
            return None;
        }

        let radius = Vector2F::splat(radius);
        options.bounds(RectF::new(center - radius, radius * 2.0))
    }

    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        if !self.attrs.display {
            return;
        }
        let options = options.apply(&self.attrs);

        let center = self.center.resolve(&options);
        let radius = self.radius.resolve(&options);

        if radius == 0.0 {
            return;
        }

        let radius = Vector2F::splat(radius);
        let mut contour = Contour::with_capacity(4);
        let tr = Transform2F::from_translation(center) * Transform2F::from_scale(radius);
        contour.push_ellipse(&tr);
        contour.close();

        let mut outline = Outline::with_capacity(1);
        outline.push_contour(contour);

        options.draw(scene, &outline);
    }
}