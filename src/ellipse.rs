use roxmltree::Node;
use svgtypes::Length;
use crate::prelude::*;
use std::str::FromStr;

use pathfinder_content::outline::{Outline, Contour};

#[derive(Debug)]
pub struct TagEllipse {
    center: Vector,
    radius: Vector,
    attrs: Attrs,
    pub id: Option<String>,
}
impl Tag for TagEllipse {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if !self.radius.has_area() || !self.attrs.display {
            return None;
        }
        let options = options.apply(&self.attrs);
        let center = options.resolve_vector(self.center);
        let radius = options.resolve_vector(self.radius);

        options.bounds(RectF::new(center - radius, radius * 2.0))
    }

    fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        if !self.radius.has_area() || !self.attrs.display {
            return;
        }
        let options = options.apply(&self.attrs);

        let center = options.resolve_vector(self.center);
        let radius = options.resolve_vector(self.radius);

        let mut contour = Contour::with_capacity(4);
        let tr = Transform2F::from_translation(center) * Transform2F::from_scale(radius);
        contour.push_ellipse(&tr);

        let mut outline = Outline::with_capacity(1);
        outline.push_contour(contour);

        options.draw(scene, &outline);
    }
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
impl TagEllipse {
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagEllipse, Error> {
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