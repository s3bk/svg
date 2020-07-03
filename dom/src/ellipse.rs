use roxmltree::Node;
use svgtypes::Length;
use crate::prelude::*;
use std::str::FromStr;


#[derive(Debug)]
pub struct TagEllipse {
    pub center: ValueVector,
    pub radius: ValueVector,
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
        parse!(node => {
            anim cx: Value<LengthX>,
            anim cy: Value<LengthY>,
            anim rx: Value<LengthX>,
            anim ry: Value<LengthY>,
            var id,
        });

        Ok(TagEllipse {
            center: ValueVector::new(cx, cy),
            radius: ValueVector::new(rx, ry),
            attrs: Attrs::parse(node)?,
            id,
        })
    }
}

#[derive(Debug)]
pub struct TagCircle {
    pub center: ValueVector,
    pub radius: Value<Length>,
    pub attrs: Attrs,
    pub id: Option<String>,
}
impl Tag for TagCircle {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
impl ParseNode for TagCircle {
    fn parse_node(node: &Node) -> Result<TagCircle, Error> {
        parse!(node => {
            anim cx: Value<LengthX>,
            anim cy: Value<LengthY>,
            anim r: Value<Length>,
            var id,
        });

        Ok(TagCircle {
            center: ValueVector::new(cx, cy),
            radius: r,
            attrs: Attrs::parse(node)?,
            id,
        })
    }
}