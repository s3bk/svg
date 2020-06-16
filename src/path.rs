use pathfinder_geometry::{
    transform2d::{Matrix2x2F},
    line_segment::LineSegment2F,
};
use pathfinder_content::{
    outline::{Outline, ArcDirection, Contour},
};
use roxmltree::{Node};
use crate::prelude::*;


#[inline]
fn reflect_on(last: Option<Vector2F>, point: Vector2F) -> Vector2F {
    match last {
        Some(c) => point * 2.0 - c,
        None => point
    }
}

#[inline]
fn safe_sqrt(x: f32) -> f32 {
    if x <= 0.0 {
        0.0
    } else {
        x.sqrt()
    }
}

#[derive(Debug)]
pub struct TagPath {
    outline: Outline,
    attrs: Attrs,
    debug: DebugInfo,
}
impl TagPath {
    pub fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(&self.attrs);
        options.draw(scene, &self.outline);

        #[cfg(feature="debug")]
        if options.debug {
            let mut options = options.clone();
            options.fill = Some(Paint::black());
            options.stroke = None;
            self.debug.draw(scene, &options);
        }
    }
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagPath, Error<'i>> {
        use std::f32::consts::PI;
        use svgtypes::{PathParser, PathSegment};

        let mut debug = DebugInfo::new();
        let mut contour = Contour::new();
        let mut outline = Outline::new();
        
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
                        let mut c2 = vec(x2, y2);
                        let mut p = vec(x, y);
                        if !abs {
                            c2 = last + c2;
                            p = last + p;
                        }
                        let c1 = reflect_on(last_cubic_control_point, p);

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
                        let mut p = vec(x, y);
                        if !abs {
                            p = last + p;
                        }
                        let c1 = reflect_on(last_quadratic_control_point, p);

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
                        //dbg!(p);
                        let x_axis_rotation = x_axis_rotation as f32 * (PI / 180.);
                        if r.x().is_finite() & r.y().is_finite() {
                            let r = r.abs();
                            let r_inv = r.inv();
                            let sign = if large_arc != sweep { 1.0 } else { -1.0 };
                            let rot = Matrix2x2F::from_rotation(x_axis_rotation);
                            //print_matrix(rot);
                            let rot_ = rot.adjugate();
                            // x'
                            let q = rot_ * (last - p) * 0.5;
                            let q2 = q * q;

                            let gamma = q2 * r_inv * r_inv;
                            let gamma = gamma.x() + gamma.y();

                            let (a, b, c) = if gamma <= 1.0 {
                                // normal case
                                let r2 = r * r;

                                let r2_prod = r2.x() * r2.y(); // r_x^2 r_y^2

                                let rq2 = r2 * q2.yx(); // (r_x^2 q_y^2, r_y^2 q_x^2)
                                let rq2_sum = rq2.x() + rq2.y(); // r_x^2 q_y^2 + r_y^2 q_x^2
                                // c'
                                let s = vec(1., -1.) * r * (q * r_inv).yx() * safe_sqrt((r2_prod - rq2_sum) / rq2_sum) * sign;
                                //dbg!(s);
                                //print_matrix(rot);
                                let c = rot * s + (last + p) * 0.5;
                                //dbg!(c);
                                let a = (q - s) * r_inv;
                                let b = -(q + s) * r_inv;
                                (a, b, c)
                            } else {
                                let c = (last + p) * 0.5;
                                let a = q * r_inv;
                                let b = -a;
                                (a, b, c)
                            };
                            
                            debug.add_point(c, "c");
                            debug.add_vector(c, a * 50., "a");
                            debug.add_vector(c, b * 50., "b");

                            let direction = match sweep {
                                false => ArcDirection::CCW,
                                true => ArcDirection::CW
                            };
                            
                            let transform = Transform2F {
                                matrix: rot,
                                vector: c
                            } * Transform2F::from_scale(r);
                            let chord = LineSegment2F::new(a, b);
                            contour.push_arc_from_unit_chord(&transform, chord, direction);
                        } else {
                            contour.push_endpoint(p);
                        }
                        last = p;
                        last_quadratic_control_point = None;
                        last_cubic_control_point = None;
                    }
                    PathSegment::ClosePath { abs }=> {
                        if last != start {
                            contour.push_endpoint(start);
                        }
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
        Ok(TagPath { outline, attrs, debug })
    }
}

fn print_matrix(mat: Matrix2x2F) {
    println!("⎛ {:6.3}  {:6.3} ⎞", mat.m11(), mat.m12());
    println!("⎝ {:6.3}  {:6.3} ⎠", mat.m21(), mat.m22());
}
