use crate::prelude::*;

use pathfinder_content::outline::{Outline, Contour};
use svgtypes::PointsParser;

#[derive(Debug)]
pub struct TagPolygon {
    points: Vec<Vector2F>,
    attrs: Attrs,
}
impl TagPolygon {
    pub fn parse<'a, 'i: 'a>(node: &Node<'a, 'i>) -> Result<TagPolygon, Error<'a>> {
        let points = node.attribute("points").map(|v| {
            PointsParser::from(v).map(|(x, y)| vec(x, y)).collect()
        }).unwrap_or_default();
        let attrs = Attrs::parse(node)?;
        Ok(TagPolygon { points, attrs })
    }

    pub fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        if self.points.len() < 2 {
            return;
        }

        let options = options.apply(&self.attrs);
        let mut contour = Contour::with_capacity(self.points.len());
        for &point in &self.points {
            contour.push_endpoint(point);
        }
        contour.close();
        let mut outline = Outline::with_capacity(1);
        outline.push_contour(contour);

        options.draw(scene, &outline);
    }
}
