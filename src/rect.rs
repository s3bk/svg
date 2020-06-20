use roxmltree::Node;
use svgtypes::Length;
use crate::prelude::*;
use std::str::FromStr;

use pathfinder_content::outline::{Outline, Contour};
use pathfinder_geometry::vector::vec2f;

fn length2vec((x, y): (Length, Length)) -> Vector2F {
    vec(x.num, y.num)
}


#[derive(Debug)]
pub struct TagRect {
    pos: (Length, Length),
    size: (Length, Length),
    radius: (Length, Length),
    attrs: Attrs,
}
impl TagRect {
    pub fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        let size = length2vec(self.size);
        if (size.x() == 0.) | (size.y() == 0.) | (!self.attrs.display) {
            return None;
        }
        let origin = length2vec(self.pos);
        let options = options.apply(&self.attrs);
        options.bounds(RectF::new(origin, size))
    }

    pub fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(&self.attrs);
        let size = length2vec(self.size);
        if (size.x() == 0.) | (size.y() == 0.) | (!self.attrs.display) {
            return;
        }

        let origin = length2vec(self.pos);
        let radius = length2vec(self.radius);
        let contour = Contour::from_rect_rounded(RectF::new(origin, size), radius);

        let mut outline = Outline::with_capacity(1);
        outline.push_contour(contour);

        options.draw(scene, &outline);
    }
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagRect, Error> {
        let x = node.attribute("x").map(Length::from_str).transpose()?.unwrap_or(Length::zero());
        let y = node.attribute("y").map(Length::from_str).transpose()?.unwrap_or(Length::zero());
        let width = node.attribute("width").map(Length::from_str).transpose()?.unwrap_or(Length::zero());
        let height = node.attribute("height").map(Length::from_str).transpose()?.unwrap_or(Length::zero());
        let rx = node.attribute("rx").map(Length::from_str).transpose()?.unwrap_or(Length::zero());
        let ry = node.attribute("ry").map(Length::from_str).transpose()?.unwrap_or(Length::zero());

        Ok(TagRect {
            pos: (x, y),
            size: (width, height),
            radius: (rx, ry),
            attrs: Attrs::parse(node)?,
        })
    }
}