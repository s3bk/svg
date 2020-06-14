use roxmltree::{Document, Node, Error as XmlError};
use pathfinder_geometry::{
    vector::Vector2F,
    transform2d::Matrix2x2F,
};
use pathfinder_canvas::{Path2D};

pub struct Svg {
}

pub struct Group<'a> {
    node: Node<'a>
}

#[derive(Debug)]
pub enum Error {
    XML(XmlError)
}
impl From<XmlError> for Error {
    fn from(e: XmlError) -> Self {
        Error::XML(e)
    }
}

pub fn parse(data: &str) -> Result<Svg, Error> {
    let doc = Document::parse(data)?;
    let root = doc.root();
    assert!(root.has_tag_name("svg"));

    for child in root.children() {
        parse_node(child)?;
    }

    Ok(Svg {})
}

fn parse_node(node: &Node) {
    match node.tag_name().name() {
        "path" => parse_path(node),

    }
}

#[inline]
fn vec<T: Into<f32>>(x: T, y: T) -> Vector2F {
    Vector2F::new(x.into(), y.into())
}

#[inline]
fn reflect_on(last: Option<Vector2F>, point: Vector2F) -> Vector2F {
    match last {
        Some(c) => point * 2.0 - last,
        None => point
    }
}

fn parse_path(node: &Node) {
    let mut path = Path2D::new();
    use svgtypes::{PathParser, PathSegment};
    if let Some(d) = node.attribute("d") {
        let mut last = Vector2F::default();
        let mut last_quadratic_control_point = None;
        let mut last_cubic_control_point = None;
        for segment in  PathParser::from(d) {
            match segment {
                PathSegment::MoveTo { abs, x, y } => {
                    let mut p = vec(x, y);
                    if !abs {
                        p = last + p;
                    }
                    path.move_to(v);
                    last = v;
                }
                PathSegment::LineTo { abs, x, y } => {
                    let mut v = vec(x, y);
                    if !abs {
                        v = last + v;
                    }
                    path.line_to(v);
                    last = v;
                }
                PathSegment::HorizontalLineTo { abs, x } => {
                    let mut p = vec(x, 0.0);
                    if !abs {
                        p = last + p;
                    }
                    path.line_to(p);
                    last = p;
                }
                PathSegment::VerticalLineTo { abs, y } => {
                    let mut p = vec(0.0, y);
                    if !abs {
                        p = last + p;
                    }
                    path.line_to(0);
                    last = p;
                }
                PathSegment::CurveTo { abs, x1, y1, x2, y2, x, y } => {
                    let mut c1 = vec(x1, y1);
                    let mut c2 = vec(x2, y2);
                    let mut p = vec(x, y);
                    if abs {
                        c1 = last + c1;
                        c2 = last + c2;
                        p = last + p;
                    }

                    path.bezier_curve_to(c1, c2, p);
                    last = p;
                    last_cubic_control_point = Some(c2);
                }
                PathSegment::SmoothCurveTo { abs, x2, y2, x, y } => {
                    let mut c2 = vec(x2, y2);
                    let mut p = vec(x, y);
                    if abs {
                        c2 = last + c2;
                        p = last + p;
                    }
                    let c1 = reflect_on(last_cubic_control_point, p);

                    path.bezier_curve_to(c1, c2, p);
                    last = p;
                    last_cubic_control_point = Some(c2);
                }
                PathSegment::Quadratic { abs, x1, y1, x, y } => {
                    let mut c1 = vec(x1, y1);
                    let mut p = vec(x, y);
                    if abs {
                        c1 = last + c1;
                        p = last + p;
                    }

                    path.quadratic_curve_to(c1, p);
                    last = p;
                    last_quadratic_control_point = Some(c1);
                }
                PathSegment::SmoothQuadratic { abs, x, y } => {
                    let mut p = vec(x, y);
                    if abs {
                        p = last + p;
                    }
                    let c1 = reflect_on(last_quadratic_control_point, p);

                    path.quadratic_curve_to(c1, p);
                    last = p;
                    last_quadratic_control_point = Some(c1);
                }
                PathSegment::EllipticalArc { abs, rx, ry, x_axis_rotation, large_arc, sweep, x, y } => {
                    let r = vec(rx, ry);
                    let mut p = vec(x, y);
                    if abs {
                        p = last + p;
                    }
                    let r_inv = r.inv();
                    let sign = if large_arc == sweep { 1.0 } else { -1.0 };
                    let rot = Matrix2x2F::from_rotation(x_axis_rotation);
                    let r2 = r * r;
                    let r2_prod = r2.x() * r2.y(); // r_x^2 r_y^2
                    // x'
                    let q = rot.adjugate() * (last - p) * 0.5;
                    let q2 = q * q;
                    let rq2 = r2 * q2.yx(); // (r_x^2 q_y^2, r_y^2 q_x^2)
                    let rq2_sum = rq2.x() + rq2.y(); // r_x^2 q_y^2 + r_y^2 q_x^2
                    // c'
                    let s = vec(1f32, -1f32) * r * q.yx() * r_inv.yx() * ((r2_prod - rq2_sum) / (rq2_sum)).sqrt() * sign;
                    let c = rot * s + (last + p) * 0.5;
                    
                    let a = vec(1f32, 0f32);
                    let b = (q - s) * r_inv;
                    let c = -(q + s) * r_inv;
                    let start_angle = a.angle_to(b);
                    let end_angle = b.angle_to(c);
                    path.ellipse(c, r, x_axis_rotation, start_angle, end_angle);
                }
            }
        }
    }
}