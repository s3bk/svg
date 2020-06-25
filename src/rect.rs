use roxmltree::Node;
use svgtypes::Length;
use crate::prelude::*;

use pathfinder_content::outline::{Outline, Contour};

#[derive(Debug)]
pub struct TagRect {
    //#[attr("x", "y", animate, default)]
    pos: ValueVector,
    
    //#[attr("width", "height", animate, default)]
    size: ValueVector,

    //#[attr("rx", "ry", animate, default)]
    radius: ValueVector,

    //#[attr("id")]
    pub id: Option<String>,

    //#[attr(other)]
    attrs: Attrs,
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

#[derive(Debug)]
struct ValueVector {
    x: Value<Length>,
    y: Value<Length>
}
impl ValueVector {
    fn new(x: Value<Length>, y: Value<Length>) -> ValueVector {
        ValueVector { x, y }
    }
    fn get(&self, options: &DrawOptions) -> Vector2F {
        let x = self.x.get(options);
        let y = self.y.get(options);
        vec2f(x, y)
    }
}

impl Tag for TagRect {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if !self.attrs.display {
            return None;
        }
        let options = options.apply(&self.attrs);

        let size = self.size.get(&options);
        if (size.x() == 0.) || (size.y() == 0.) {
            return None;
        }
        
        let origin = self.pos.get(&options);
        options.bounds(RectF::new(origin, size))
    }

    fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        if !self.attrs.display {
            return;
        }
        let options = options.apply(&self.attrs);

        let size = self.size.get(&options);
        if (size.x() == 0.) || (size.y() == 0.) {
            return;
        }
        
        let origin = self.pos.get(&options);
        let radius = self.radius.get(&options);
        let contour = Contour::from_rect_rounded(RectF::new(origin, size), radius);

        let mut outline = Outline::with_capacity(1);
        outline.push_contour(contour);

        options.draw(scene, &outline);
    }
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
impl TagRect {
    pub fn parse(node: &Node) -> Result<TagRect, Error> {
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