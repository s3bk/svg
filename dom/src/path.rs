use pathfinder_geometry::{
    transform2d::{Matrix2x2F},
};
use pathfinder_content::{
    outline::{Outline, ArcDirection, Contour},
};
use roxmltree::{Node};
use crate::prelude::*;
use crate::parse_element;


#[inline]
fn reflect_on(last: Option<Vector2F>, point: Vector2F) -> Vector2F {
    match last {
        Some(c) => point * 2.0 - c,
        None => point
    }
}

#[derive(Debug)]
pub struct TagClipPath {
    pub id: Option<String>,
    pub items: Vec<Item>,
}
impl Tag for TagClipPath {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
impl ParseNode for TagClipPath {
    fn parse_node(node: &Node) -> Result<TagClipPath, Error> {
        let id = node.attribute("id").map(From::from);
        let mut items = Vec::with_capacity(1);
        for elem in node.children().filter(|n| n.is_element()) {
            if let Some(item) = parse_element(&elem)? {
                items.push(item);
            }
        }
        Ok(TagClipPath { id, items })
    }
}

#[derive(Debug)]
pub struct TagPath {
    pub outline: Outline,
    pub attrs: Attrs,
    pub id: Option<String>,
}
impl Tag for TagPath {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
impl ParseNode for TagPath {
    fn parse_node(node: &Node) -> Result<TagPath, Error> {
        use std::f32::consts::PI;
        use svgtypes::{PathParser, PathSegment};

        let mut contour = Contour::new();
        let mut outline = Outline::new();
        let id = node.attribute("id").map(|s| s.into());
        
        if let Some(d) = node.attribute("d") {
            let mut start = Vector2F::default();
            let mut last = Vector2F::default();
            let mut last_quadratic_control_point = None;
            let mut last_cubic_control_point = None;
            for segment in PathParser::from(d) {
                match segment? {
                    PathSegment::MoveTo { abs, x, y } => {
                        let mut p = vec(x, y);
                        if !abs {
                            p = last + p;
                        }
                        if !contour.is_empty() {
                            outline.push_contour(contour.clone());
                            contour.clear();
                        }
                        contour.push_endpoint(p);
                        last = p;
                        last_quadratic_control_point = None;
                        last_cubic_control_point = None;
                        start = p;
                    }
                    PathSegment::LineTo { abs, x, y } => {
                        let mut p = vec(x, y);
                        if !abs {
                            p = last + p;
                        }
                        contour.push_endpoint(p);
                        last = p;
                        last_quadratic_control_point = None;
                        last_cubic_control_point = None;
                    }
                    PathSegment::HorizontalLineTo { abs, x } => {
                        let p = if abs {
                            Vector2F::new(x as f32, last.y())
                        } else {
                            Vector2F::new(x as f32, 0.0) + last
                        };
                        contour.push_endpoint(p);
                        last = p;
                        last_quadratic_control_point = None;
                        last_cubic_control_point = None;
                    }
                    PathSegment::VerticalLineTo { abs, y } => {
                        let p = if abs {
                            Vector2F::new(last.x(), y as f32)
                        } else {
                            Vector2F::new(0.0, y as f32) + last
                        };
                        contour.push_endpoint(p);
                        last = p;
                        last_quadratic_control_point = None;
                        last_cubic_control_point = None;
                    }
                    PathSegment::CurveTo { abs, x1, y1, x2, y2, x, y } => {
                        let mut c1 = vec(x1, y1);
                        let mut c2 = vec(x2, y2);
                        let mut p = vec(x, y);
                        if !abs {
                            c1 = last + c1;
                            c2 = last + c2;
                            p = last + p;
                        }

                        contour.push_cubic(c1, c2, p);
                        last = p;
                        last_quadratic_control_point = None;
                        last_cubic_control_point = Some(c2);
                    }
                    PathSegment::SmoothCurveTo { abs, x2, y2, x, y } => {
                        let c1 = reflect_on(last_cubic_control_point, last);
                        let mut c2 = vec(x2, y2);
                        let mut p = vec(x, y);
                        if !abs {
                            c2 = last + c2;
                            p = last + p;
                        }

                        contour.push_cubic(c1, c2, p);
                        last = p;
                        last_quadratic_control_point = None;
                        last_cubic_control_point = Some(c2);
                    }
                    PathSegment::Quadratic { abs, x1, y1, x, y } => {
                        let mut c1 = vec(x1, y1);
                        let mut p = vec(x, y);
                        if !abs {
                            c1 = last + c1;
                            p = last + p;
                        }

                        contour.push_quadratic(c1, p);
                        last = p;
                        last_quadratic_control_point = Some(c1);
                        last_cubic_control_point = None;
                    }
                    PathSegment::SmoothQuadratic { abs, x, y } => {
                        let c1 = reflect_on(last_quadratic_control_point, last);
                        let mut p = vec(x, y);
                        if !abs {
                            p = last + p;
                        }

                        contour.push_quadratic(c1, p);
                        last = p;
                        last_quadratic_control_point = Some(c1);
                        last_cubic_control_point = None;
                    }
                    PathSegment::EllipticalArc { abs, rx, ry, x_axis_rotation, large_arc, sweep, x, y } => {
                        let r = vec(rx, ry);
                        let mut p = vec(x, y);
                        if !abs {
                            p = last + p;
                        }

                        if p == last {
                            println!("arc over zero distance");
                        } else {
                            let direction = match sweep {
                                false => ArcDirection::CCW,
                                true => ArcDirection::CW
                            };
                            contour.push_svg_arc(r, x_axis_rotation as f32 * (PI / 180.), large_arc, direction, p);
                        }
                        last = p;
                        last_quadratic_control_point = None;
                        last_cubic_control_point = None;
                    }
                    PathSegment::ClosePath { abs }=> {
                        if last != start {
                            contour.push_endpoint(start);
                        }
                        last = start;
                        contour.close();
                    }
                }
            }
            if !contour.is_empty() {
                outline.push_contour(contour.clone());
                contour.clear();
            }
        }

        let attrs = Attrs::parse(node)?;
        Ok(TagPath { id, outline, attrs })
    }
}
