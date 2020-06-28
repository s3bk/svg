use roxmltree::Node;
use svgtypes::Length;
use crate::prelude::*;

use pathfinder_content::outline::{Outline, Contour};

#[derive(Debug)]
pub struct TagRect {
    //#[attr("x", "y", animate, default)]
    pub pos: ValueVector,
    
    //#[attr("width", "height", animate, default)]
    pub size: ValueVector,

    //#[attr("rx", "ry", animate, default)]
    pub radius: ValueVector,

    //#[attr("id")]
    pub id: Option<String>,

    //#[attr(other)]
    pub attrs: Attrs,
}
/*
impl TagRect {
    pub fn parse(node: &Node) -> Result<TagRect, Error> {
        let mut attr_x = parse_or_default(node.attribute("x"))?;
        let mut attr_y = parse_or_default(node.attribute("y"))?;
        let id: Option<String> = parse(node.attribute("id"))?;

        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "animate" | "animateColor" => match n.attribute("attributeName").unwrap() {
                    "x" => x.parse_animate_node(&n)?,
                    "y" => y.parse_animate_node(&n)?,
                    "width" => width.parse_animate_node(&n)?,
                    "height" => height.parse_animate_node(&n)?,
                    "rx" => rx.parse_animate_node(&n)?,
                    "ry" => ry.parse_animate_node(&n)?,
                    _ => {}
                }
                _ => {}
            }
        }

        let mut pos: ValueVector = ValueVector::new(attr_x, attr_y);
        TagRect {
            pos,
            ..
            id,
            attrs: Attrs::parse(node)?,
        }
        Ok(TagRect {
            pos: ValueVector::new(x, y),
            size: ValueVector::new(width, height),
            radius: ValueVector::new(rx, ry),
            attrs,
            id,
        })
    }
}*/


impl Tag for TagRect {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
impl ParseNode for TagRect {
    fn parse_node(node: &Node) -> Result<TagRect, Error> {
        let mut x = Value::parse_or_default(node.attribute("x"))?;
        let mut y = Value::parse_or_default(node.attribute("y"))?;
        let mut width = Value::parse_or_default(node.attribute("width"))?;
        let mut height = Value::parse_or_default(node.attribute("height"))?;
        let mut rx = Value::parse_or_default(node.attribute("rx"))?;
        let mut ry = Value::parse_or_default(node.attribute("ry"))?;
        let id = node.attribute("id").map(|s| s.into());
        let attrs = Attrs::parse(node)?;

        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "animate" | "animateColor" => match n.attribute("attributeName").unwrap() {
                    "x" => x.parse_animate_node(&n)?,
                    "y" => y.parse_animate_node(&n)?,
                    "width" => width.parse_animate_node(&n)?,
                    "height" => height.parse_animate_node(&n)?,
                    "rx" => rx.parse_animate_node(&n)?,
                    "ry" => ry.parse_animate_node(&n)?,
                    _ => {}
                }
                _ => {}
            }
        }
        Ok(TagRect {
            pos: ValueVector::new(x, y),
            size: ValueVector::new(width, height),
            radius: ValueVector::new(rx, ry),
            attrs,
            id,
        })
    }
}