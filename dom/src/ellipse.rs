use roxmltree::Node;
use svgtypes::Length;
use crate::prelude::*;
use std::str::FromStr;

use pathfinder_content::outline::{Outline, Contour};

#[derive(Debug)]
pub struct TagEllipse {
    pub center: Vector,
    pub radius: Vector,
    pub attrs: Attrs,
    pub id: Option<String>,
}
impl Tag for TagEllipse {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
impl ParseNode for TagEllipse {
    fn parse_node(node: &Node) -> Result<TagEllipse, Error> {
        let cx = node.attribute("cx").map(Length::from_str).transpose()?.unwrap_or(Length::zero());
        let cy = node.attribute("cy").map(Length::from_str).transpose()?.unwrap_or(Length::zero());
        let (rx, ry) = match node.tag_name().name() {
            "circle" => {
                let r = Length::from_str(node.attribute("r").ok_or_else(|| Error::MissingAttribute("r".into()))?)?;
                (r, r)
            }
            "ellipse" => {
                let rx = Length::from_str(node.attribute("rx").ok_or_else(|| Error::MissingAttribute("rx".into()))?)?;
                let ry = Length::from_str(node.attribute("ry").ok_or_else(|| Error::MissingAttribute("ry".into()))?)?;
                (rx, ry)
            },
            _ => unreachable!()
        };
        let id = node.attribute("id").map(|s| s.into());

        Ok(TagEllipse {
            center: Vector(cx, cy),
            radius: Vector(rx, ry),
            attrs: Attrs::parse(node)?,
            id,
        })
    }
}
