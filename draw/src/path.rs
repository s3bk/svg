use pathfinder_content::outline::Outline;
use crate::prelude::*;

impl Resolve for TagClipPath {
    type Output = Outline;
    fn resolve(&self, options: &Options) -> Outline {
        let mut outline = Outline::new();
        for item in &self.items {
            let o = match item {
                Item::Path(path) => path.outline(options),
                Item::Rect(rect) => rect.outline(options),
                Item::Circle(circle) => circle.outline(options),
                Item::Polygon(polygon) => polygon.outline(options),
                _ => None
            };
            if let Some(o) = o {
                outline.push_outline(o)
            }
        }
        outline
    }
}

impl Shape for TagPath {
    fn outline(&self, options: &Options) -> Option<Outline> {
        let options = options.apply(&self.attrs);
        Some(self.outline.clone().transformed(options.get_transform()))
    }
}


impl DrawItem for TagPath {
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