use crate::prelude::*;

use pathfinder_content::outline::{Outline, Contour};
use svgtypes::PointsParser;

#[derive(Debug)]
pub struct TagPolygon {
    outline: Outline,
    attrs: Attrs,
    pub id: Option<String>,
}
impl Tag for TagPolygon {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if self.attrs.display && self.outline.len() > 0 {
            let options = options.apply(&self.attrs);
            options.bounds(self.outline.bounds())
        } else {
            None
        }
    }
    fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(&self.attrs);
        options.draw(scene, &self.outline);
    }
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
impl TagPolygon {
    pub fn parse<'a, 'i: 'a>(node: &Node<'a, 'i>) -> Result<TagPolygon, Error> {
        let mut contour = Contour::new();
        if let Some(v) = node.attribute("points") {
            for (x, y) in PointsParser::from(v) {
                contour.push_endpoint(vec(x, y));
            }
        }

        let mut outline = Outline::with_capacity(1);
        outline.push_contour(contour);
        
        let attrs = Attrs::parse(node)?;
        let id = node.attribute("id").map(|s| s.into());
        Ok(TagPolygon { id, outline, attrs })
    }
}
