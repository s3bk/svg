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
    pub rx: Value<Option<Length>>,
    pub ry: Value<Option<Length>>,

    //#[attr("id")]
    pub id: Option<String>,

    //#[attr(other)]
    pub attrs: Attrs,
}


impl Tag for TagRect {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}


impl ParseNode for TagRect {
    fn parse_node(node: &Node) -> Result<TagRect, Error> {
        parse!(node => {
            anim x: Value<LengthX>,
            anim y: Value<LengthY>,
            anim height: Value<LengthY>,
            anim width: Value<LengthX>,
            anim rx: Value<Option<Length>>,
            anim ry: Value<Option<Length>>,
            var id,
        });
        let attrs = Attrs::parse(node)?;
        Ok(TagRect {
            pos: ValueVector::new(x, y),
            size: ValueVector::new(width, height),
            rx, ry,
            attrs,
            id,
        })
    }
}