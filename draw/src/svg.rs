use crate::prelude::*;

impl DrawItem for TagSvg {
    fn bounds(&self, options: &BoundsOptions) -> Option<RectF> {
        self.view_box.as_ref().map(|r| r.resolve(options))
        .or_else(|| max_bounds(self.items.iter().flat_map(|item| item.bounds(&options))))
    }
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let mut options = options.apply(scene, &self.attrs);
        if let Some(ref view_box) = self.view_box {
            options.apply_viewbox(self.width, self.height, view_box);
        }
        for item in self.items.iter() {
            item.draw_to(scene, &options);
        }
    }
}
