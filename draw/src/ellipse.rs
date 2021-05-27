use crate::prelude::*;
use pathfinder_content::outline::{Outline, Contour};

fn ellipse_outline(tag: &TagEllipse, options: &Options) -> Option<Outline> {
    let center = tag.center.resolve(&options);
    let radius = tag.radius.resolve(&options);

    if radius.x() == 0.0 || radius.y() == 0.0 {
        return None;
    }

    let mut contour = Contour::with_capacity(4);
    let tr = Transform2F::from_translation(center) * Transform2F::from_scale(radius);
    contour.push_ellipse(&tr);

    let mut outline = Outline::with_capacity(1);
    outline.push_contour(contour);
    Some(outline)
}

impl Shape for TagEllipse {
    fn outline(&self, options: &Options) -> Option<Outline> {
        if !self.attrs.display {
            return None;
        }
        let options = options.apply(&self.attrs);
        ellipse_outline(self, &options).map(|o| o.transformed(options.get_transform()))
    }
}
impl DrawItem for TagEllipse {
    fn bounds(&self, options: &BoundsOptions) -> Option<RectF> {
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
        let options = options.apply(scene, &self.attrs);

        if let Some(outline) = ellipse_outline(self, &options) {
            options.draw(scene, &outline);
        }
    }
}

fn circle_outline(tag: &TagCircle, options: &Options) -> Option<Outline> {
    let center = tag.center.resolve(&options);
    let radius = tag.radius.resolve(&options);

    if radius == 0.0 {
        return None;
    }

    let radius = Vector2F::splat(radius);
    let mut contour = Contour::with_capacity(4);
    let tr = Transform2F::from_translation(center) * Transform2F::from_scale(radius);
    contour.push_ellipse(&tr);

    let mut outline = Outline::with_capacity(1);
    outline.push_contour(contour);
    Some(outline)
}
impl Shape for TagCircle {
    fn outline(&self, options: &Options) -> Option<Outline> {
        if !self.attrs.display {
            return None;
        }
        let options = options.apply(&self.attrs);
        circle_outline(self, &options).map(|o| o.transformed(options.get_transform()))
    }
}
impl DrawItem for TagCircle {
    fn bounds(&self, options: &BoundsOptions) -> Option<RectF> {
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
        let options = options.apply(scene, &self.attrs);

        if let Some(outline) = circle_outline(self, &options) {
            options.draw(scene, &outline);
        }
    }
}