use crate::prelude::*;
use pathfinder_content::gradient::{Gradient, GradientGeometry, ColorStop};
use pathfinder_color::{ColorU, ColorF};
use pathfinder_geometry::line_segment::LineSegment2F;
use pathfinder_simd::default::F32x2;
use svgtypes::Color;
use crate::merge;

#[derive(Debug)]
pub struct TagLinearGradient {
    pub from: (Option<Length>, Option<Length>),
    pub to: (Option<Length>, Option<Length>),
    pub gradient_transform: Option<Transform2F>,
    pub stops: Vec<TagStop>,
    pub id: Option<String>,
    xlink_href: Option<String>,
}
struct PartialLinearGradient<'a> {
    from: (Option<Length>, Option<Length>),
    to: (Option<Length>, Option<Length>),
    gradient_transform: Option<Transform2F>,
    stops: &'a [TagStop],
}

#[derive(Debug)]
pub struct TagRadialGradient {
    pub center: (Option<Length>, Option<Length>),
    pub focus: (Option<Length>, Option<Length>),
    pub radius: Option<Length>,
    pub gradient_transform: Option<Transform2F>,
    pub stops: Vec<TagStop>,
    pub id: Option<String>,
    xlink_href: Option<String>,
}
struct PartialRadialGradient<'a> {
    center: (Option<Length>, Option<Length>),
    focus: (Option<Length>, Option<Length>),
    radius: Option<Length>,
    gradient_transform: Option<Transform2F>,
    stops: &'a [TagStop],
}

#[derive(Debug)]
pub struct TagStop {
    pub offset: f32,
    pub color: Color,
    pub opacity: f32,
}

fn href(node: &Node) -> Option<String> {
    let xlink = node.lookup_namespace_uri(Some("xlink")).unwrap_or_default();
    node.attribute((xlink, "href")).map(|s| s.to_owned())
}

impl TagLinearGradient {
    pub fn parse<'a, 'i: 'a>(node: &Node<'a, 'i>) -> Result<TagLinearGradient, Error> {
        let x1 = node.attribute("x1").map(Length::from_str).transpose()?;
        let y1 = node.attribute("y1").map(Length::from_str).transpose()?;
        let x2 = node.attribute("x2").map(Length::from_str).transpose()?;
        let y2 = node.attribute("y2").map(Length::from_str).transpose()?;
        let gradient_transform = node.attribute("gradientTransform").map(transform_list).transpose()?;
        let id = node.attribute("id").map(|s| s.to_owned());
        let xlink_href = href(node);
    
        let mut stops = Vec::new();
        for elem in node.children().filter(|n| n.is_element()) {
            match elem.tag_name().name() {
                "stop" => stops.push(TagStop::parse(&elem)?),
                _ => {}
            }
        }
    
        Ok(TagLinearGradient {
            from: (x1, y1),
            to: (x2, y2),
            gradient_transform,
            stops,
            id,
            xlink_href
        })
    }


    pub fn build(&self, options: &DrawOptions, opacity: f32) -> Gradient {
        if let Some(item) = self.xlink_href.as_ref().and_then(|href| options.ctx.resolve(&href[1..])) {
            match &**item {
                Item::LinearGradient(other) => {
                    return PartialLinearGradient {
                        from: merge_point(&self.from, &other.from),
                        to: merge_point(&self.to, &other.to),
                        gradient_transform: merge(&self.gradient_transform, &other.gradient_transform),
                        stops: select_stops(&self.stops, &other.stops)
                    }.build(options, opacity)
                },
                Item::RadialGradient(other) => {
                    return PartialLinearGradient {
                        from: self.from,
                        to: self.to,
                        gradient_transform: self.gradient_transform,
                        stops: select_stops(&self.stops, &other.stops)
                    }.build(options, opacity)
                },
                _ => {}
            }
        }

        PartialLinearGradient {
            from: self.from,
            to: self.to,
            gradient_transform: self.gradient_transform,
            stops: &self.stops
        }.build(options, opacity)
    }
}

fn select_stops<'a>(a: &'a [TagStop], b: &'a [TagStop]) -> &'a [TagStop] {
    if a.len() > 0 {
        a
    } else {
        b
    }
}

fn merge_point(
    a: &(Option<Length>, Option<Length>),
    b: &(Option<Length>, Option<Length>)
) -> (Option<Length>, Option<Length>) {
    (
        merge(&a.0, &b.0),
        merge(&a.1, &b.1)
    )
}
fn length_or_percent(a: Option<Length>, default: f64) -> Length {
    match a {
        Some(l) => l,
        None => Length::new(default, LengthUnit::Percent)
    }
}
fn point_or_percent(a: (Option<Length>, Option<Length>), default: (f64, f64)) -> (Length, Length) {
    (
        length_or_percent(a.0, default.0),
        length_or_percent(a.1, default.1),
    )
}

impl TagRadialGradient {
    pub fn parse<'a, 'i: 'a>(node: &Node<'a, 'i>) -> Result<TagRadialGradient, Error> {
        let cx = node.attribute("cx").map(Length::from_str).transpose()?;
        let cy = node.attribute("cy").map(Length::from_str).transpose()?;
        let r = node.attribute("r").map(Length::from_str).transpose()?;
        let fx = node.attribute("x2").map(Length::from_str).transpose()?;
        let fy = node.attribute("y2").map(Length::from_str).transpose()?;
        let gradient_transform = node.attribute("gradientTransform").map(transform_list).transpose()?;
        let id = node.attribute("id").map(|s| s.to_owned());
        let xlink_href = href(node);
    
        let mut stops = Vec::new();
        for elem in node.children().filter(|n| n.is_element()) {
            match elem.tag_name().name() {
                "stop" => stops.push(TagStop::parse(&elem)?),
                _ => {}
            }
        }
    
        Ok(TagRadialGradient {
            center: (cx, cy),
            focus: (fx, fy),
            radius: r,
            gradient_transform,
            stops,
            id,
            xlink_href,
        })
    }

    pub fn build(&self, options: &DrawOptions, opacity: f32) -> Gradient {
        if let Some(item) = self.xlink_href.as_ref().and_then(|href| options.ctx.resolve(&href[1..])) {
            match &**item {
                Item::RadialGradient(ref other) => {
                    return PartialRadialGradient {
                        center: merge_point(&self.center, &other.center),
                        focus: merge_point(&self.focus, &other.focus),
                        radius: self.radius.or(other.radius),
                        gradient_transform: self.gradient_transform.or(other.gradient_transform),
                        stops: select_stops(&self.stops, &other.stops)
                    }.build(options, opacity)
                }
                Item::LinearGradient(ref other) => {
                    return PartialRadialGradient {
                        center: self.center,
                        focus: self.focus,
                        radius: self.radius,
                        gradient_transform: self.gradient_transform,
                        stops: select_stops(&self.stops, &other.stops)
                    }.build(options, opacity)
                }
                _ => {}
            }
        }
        PartialRadialGradient {
            center: self.center,
            focus: self.focus,
            radius: self.radius,
            gradient_transform: self.gradient_transform,
            stops: &self.stops
        }.build(options, opacity)
    }
}

impl<'a> PartialLinearGradient<'a> {
    fn build(self, options: &DrawOptions, opacity: f32) -> Gradient {
        let from = point_or_percent(self.from, (0., 0.));
        let to = point_or_percent(self.to, (100., 0.));
        let gradient_transform = self.gradient_transform.unwrap_or_default();

        let mut gradient = Gradient::linear_from_points(
            options.resolve_point(from),
            options.resolve_point(to)
        );
        for stop in self.stops {
            gradient.add_color_stop(stop.color_u(opacity), stop.offset);
        }

        gradient.apply_transform(options.transform * gradient_transform);
        gradient
    }
}
impl<'a> PartialRadialGradient<'a> {
    fn build(&self, options: &DrawOptions, opacity: f32) -> Gradient {
        let center = point_or_percent(self.center, (50., 50.));
        let focus = (self.focus.0.unwrap_or(center.0), self.focus.1.unwrap_or(center.1));
        let radius = length_or_percent(self.radius, 50.);
        let gradient_transform = self.gradient_transform.unwrap_or_default();

        let mut gradient = Gradient::radial(
            LineSegment2F::new(
                options.resolve_point(focus),
                options.resolve_point(center)
            ),
            F32x2::new(0.0, options.resolve_length(radius))
        );
        for stop in self.stops {
            gradient.add_color_stop(stop.color_u(opacity), stop.offset);
        }

        gradient.apply_transform(options.transform * gradient_transform);
        gradient
    }
}

fn number_or_percent(s: &str) -> Result<f32, Error> {
    match Length::from_str(s)? {
        Length { num, unit: LengthUnit::None } => Ok(num as f32),
        Length { num, unit: LengthUnit::Percent } => Ok(0.01 * num as f32),
        _ => Err(Error::InvalidAttributeValue("number or percent".into()))
    }
}


impl TagStop {
    fn new() -> TagStop {
        TagStop { offset: 0.0, color: Color::black(), opacity: 1.0 }
    }

    fn apply<'a>(&mut self, key: &'a str, val: &'a str) -> Result<(), Error> {
        match key {
            "offset" => self.offset = number_or_percent(val)?,
            "stop-opacity" => self.opacity = opacity(val)?,
            "stop-color" => self.color = Color::from_str(val)?,
            "style" => {
                for (key, val) in style_list(val) {
                    self.apply(key, val)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn parse<'a, 'i: 'a>(node: &Node<'a, 'i>) -> Result<TagStop, Error> {
        let mut stop = TagStop::new();

        for attr in node.attributes() {
            stop.apply(attr.name(), attr.value());
        }

        Ok(stop)
    }

    pub fn color_u(&self, opacity: f32) -> ColorU {
        let Color { red, green, blue } = self.color;
        let alpha = (opacity * self.opacity * 255.) as u8;
        ColorU::new(red, green, blue, alpha)
    }
}

