use roxmltree::Node;
use svgtypes::Length;
use crate::prelude::*;
use std::str::FromStr;

use pathfinder_content::outline::{Outline, Contour};
use pathfinder_geometry::vector::vec2f;

fn length2vec((x, y): (Length, Length)) -> Vector2F {
    vec(x.num, y.num)
}

use std::f32::consts::SQRT_2;
const QUARTER_ARC_CP_FROM_OUTSIDE: f32 = (3.0 - 4.0 * (SQRT_2 - 1.0)) / 3.0;

#[derive(Debug)]
pub struct TagRect {
    pos: (Length, Length),
    size: (Length, Length),
    radius: (Length, Length),
    attrs: Attrs,
}
impl TagRect {
    pub fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(&self.attrs);

        let size = length2vec(self.size);
        if (size.x() == 0.) | (size.y() == 0.) {
            return;
        }
        let origin = length2vec(self.pos);
        let radius = length2vec(self.radius);
        let outer_rect = RectF::new(origin, size);

        let contour = if radius.is_zero() {
            Contour::from_rect(outer_rect)
        } else {
            let radius = radius.min(size * 0.5);
            let contol_point_offset = radius * QUARTER_ARC_CP_FROM_OUTSIDE;

            let mut contour = Contour::with_capacity(8);

            // upper left corner
            {
                let p0 = outer_rect.origin();
                let p1 = p0 + contol_point_offset;
                let p2 = p0 + radius;
                contour.push_endpoint(vec2f(p0.x(), p2.y()));
                contour.push_cubic(
                    vec2f(p0.x(), p1.y()),
                    vec2f(p1.x(), p0.y()),
                    vec2f(p2.x(), p0.y())
                );
            }

            // upper right
            {
                let p0 = outer_rect.upper_right();
                let p1 = p0 + contol_point_offset * vec2f(-1.0, 1.0);
                let p2 = p0 + radius * vec2f(-1.0, 1.0);
                contour.push_endpoint(vec2f(p2.x(), p0.y()));
                contour.push_cubic(
                    vec2f(p1.x(), p0.y()),
                    vec2f(p0.x(), p1.y()),
                    vec2f(p0.x(), p2.y())
                );
            }

            // lower right
            {
                let p0 = outer_rect.lower_right();
                let p1 = p0 + contol_point_offset * vec2f(-1.0, -1.0);
                let p2 = p0 + radius * vec2f(-1.0, -1.0);
                contour.push_endpoint(vec2f(p0.x(), p2.y()));
                contour.push_cubic(
                    vec2f(p0.x(), p1.y()),
                    vec2f(p1.x(), p0.y()),
                    vec2f(p2.x(), p0.y())
                );
            }

            // lower left
            {
                let p0 = outer_rect.lower_left();
                let p1 = p0 + contol_point_offset * vec2f(1.0, -1.0);
                let p2 = p0 + radius * vec2f(1.0, -1.0);
                contour.push_endpoint(vec2f(p2.x(), p0.y()));
                contour.push_cubic(
                    vec2f(p1.x(), p0.y()),
                    vec2f(p0.x(), p1.y()),
                    vec2f(p0.x(), p2.y())
                );
            }

            contour.close();
            contour
        };

        let mut outline = Outline::with_capacity(1);
        outline.push_contour(contour);

        options.draw(scene, &outline);
    }
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagRect, Error<'i>> {
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