use pathfinder_content::outline::Outline;
use crate::prelude::*;
use crate::rect::rect_outline;

impl Resolve for TagClipPath {
    type Output = Outline;
    fn resolve(&self, options: &DrawOptions) -> Outline {
        let mut outline = Outline::new();
        for item in &self.items {
            match item {
                Item::Path(path) => {
                    let tr = options.transform * path.attrs.transform.resolve(options);
                    outline.push_outline(path.outline.clone().transformed(&tr));
                }
                Item::Rect(rect) => {
                    let tr = options.transform * rect.attrs.transform.resolve(options);
                    if let Some((_, o)) = rect_outline(rect, options) {
                        outline.push_outline(o.transformed(&tr));
                    }
                }
                _ => {}
            }
        }
        outline
    }
}


impl DrawItem for TagPath {
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