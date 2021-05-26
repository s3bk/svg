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

#[derive(Debug)]
pub struct TagPolyline {
    pub outline: Outline,
    pub attrs: Attrs,
    pub id: Option<String>,
}
impl Tag for TagPolyline {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
impl ParseNode for TagPolyline {
    fn parse_node(node: &Node) -> Result<TagPolyline, Error> {
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
        Ok(TagPolyline { id, outline, attrs })
    }
}

#[derive(Debug)]
pub struct TagLine {
    pub p1: ValueVector,
    pub p2: ValueVector,
    pub attrs: Attrs,
    pub id: Option<String>,
}
impl Tag for TagLine {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
impl ParseNode for TagLine {
    fn parse_node(node: &Node) -> Result<TagLine, Error> {
        parse!(node => {
            anim x1: Value<LengthX>,
            anim y1: Value<LengthY>,
            anim x2: Value<LengthX>,
            anim y2: Value<LengthY>,
            var id,
        });
        
        let attrs = Attrs::parse(node)?;
        Ok(TagLine {
            id,
            p1: ValueVector::new(x1, y1),
            p2: ValueVector::new(x2, y2),
            attrs
        })
    }
}
