use crate::prelude::*;

use std::sync::Arc;

impl DrawItem for TagSvg {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        self.view_box.as_ref().map(|r| options.resolve_rect(r))
        .or_else(|| max_bounds(self.items.iter().flat_map(|item| item.bounds(&options))))
    }
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let mut options = options.apply(&self.attrs);
        if let Some(ref view_box) = self.view_box {
            let size = Vector(
                self.width.unwrap_or(view_box.width),
                self.height.unwrap_or(view_box.height)
            );
            options.apply_viewbox(size, view_box);
        }
        for item in self.items.iter() {
            item.draw_to(scene, &options);
        }
    }
}
