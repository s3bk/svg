use pathfinder_content::outline::Outline;
use crate::prelude::*;

impl Resolve for TagClipPath {
    type Output = Outline;
    fn resolve(&self, options: &DrawOptions) -> Outline {
        let mut outline = Outline::new();
        for path in &self.paths {
            let tr = options.transform * path.attrs.transform.resolve(options);
            outline.merge(path.outline.clone().transformed(&tr));
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

        #[cfg(feature="debug")]
        if options.debug {
            let mut options = options.clone();
            options.fill = Some(Paint::black());
            options.stroke = None;
            self.debug.draw(scene, &options);
        }
    }
}