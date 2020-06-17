use crate::prelude::*;

use pathfinder_content::outline::{Outline, Contour};
use svgtypes::PointsParser;

#[derive(Debug)]
pub struct TagPolygon {
    outline: Outline,
    attrs: Attrs,
}
impl TagPolygon {
    pub fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if self.attrs.display && self.outline.len() > 0 {
            let options = options.apply(&self.attrs);
            options.bounds(self.outline.bounds())
        } else {
            None
        }
    }
    pub fn parse<'a, 'i: 'a>(node: &Node<'a, 'i>) -> Result<TagPolygon, Error<'a>> {
        let mut contour = Contour::new();
        if let Some(v) = node.attribute("points") {
            for (x, y) in PointsParser::from(v) {
                contour.push_endpoint(vec(x, y));
            }
        }

        let mut outline = Outline::with_capacity(1);
        outline.push_contour(contour);
        
        let attrs = Attrs::parse(node)?;
        Ok(TagPolygon { outline, attrs })
    }

    pub fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(&self.attrs);
        options.draw(scene, &self.outline);
    }
}
