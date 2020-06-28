use crate::prelude::*;

use pathfinder_content::outline::{Outline, Contour};
use svgtypes::PointsParser;

#[derive(Debug)]
pub struct TagPolygon {
    pub outline: Outline,
    pub attrs: Attrs,
    pub id: Option<String>,
}
impl Tag for TagPolygon {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
impl ParseNode for TagPolygon {
    fn parse_node(node: &Node) -> Result<TagPolygon, Error> {
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
