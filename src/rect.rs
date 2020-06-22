use roxmltree::Node;
use svgtypes::Length;
use crate::prelude::*;
use std::str::FromStr;

use pathfinder_content::outline::{Outline, Contour};

#[derive(Debug)]
pub struct TagRect {
    pos: (Length, Length),
    size: (Length, Length),
    radius: (Length, Length),
    attrs: Attrs,
    pub id: Option<String>,
}
impl Tag for TagRect {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        let (w, h) = self.size;
        if (w.num == 0.) | (h.num == 0.) | (!self.attrs.display) {
            return None;
        }
        let options = options.apply(&self.attrs);
        
        let origin = options.resolve_point(self.pos);
        let size = options.resolve_point(self.size);
        options.bounds(RectF::new(origin, size))
    }

    fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let (w, h) = self.size;
        if (w.num == 0.) | (h.num == 0.) | (!self.attrs.display) {
            return;
        }
        let options = options.apply(&self.attrs);

        let origin = options.resolve_point(self.pos);
        let size = options.resolve_point(self.size);
        let radius = options.resolve_point(self.radius);
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
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagRect, Error> {
        let x = node.attribute("x").map(Length::from_str).transpose()?.unwrap_or(Length::zero());
        let y = node.attribute("y").map(Length::from_str).transpose()?.unwrap_or(Length::zero());
        let width = node.attribute("width").map(Length::from_str).transpose()?.unwrap_or(Length::zero());
        let height = node.attribute("height").map(Length::from_str).transpose()?.unwrap_or(Length::zero());
        let rx = node.attribute("rx").map(Length::from_str).transpose()?.unwrap_or(Length::zero());
        let ry = node.attribute("ry").map(Length::from_str).transpose()?.unwrap_or(Length::zero());
        let id = node.attribute("id").map(|s| s.into());
        let attrs = Attrs::parse(node)?;

        Ok(TagRect {
            pos: (x, y),
            size: (width, height),
            radius: (rx, ry),
            attrs,
            id,
        })
    }
}