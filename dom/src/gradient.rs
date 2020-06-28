use crate::prelude::*;
use pathfinder_content::gradient::{Gradient};
use pathfinder_color::{ColorU};
use pathfinder_geometry::line_segment::LineSegment2F;
use pathfinder_simd::default::F32x2;
use svgtypes::Color;

#[derive(Debug)]
pub struct TagLinearGradient {
    pub from: (Option<Length>, Option<Length>),
    pub to: (Option<Length>, Option<Length>),
    pub gradient_transform: Option<Transform2F>,
    pub stops: Vec<TagStop>,
    pub id: Option<String>,
    pub href: Option<String>,
}

#[derive(Debug)]
pub struct TagRadialGradient {
    pub center: (Option<Length>, Option<Length>),
    pub focus: (Option<Length>, Option<Length>),
    pub radius: Option<Length>,
    pub gradient_transform: Option<Transform2F>,
    pub stops: Vec<TagStop>,
    pub id: Option<String>,
    pub href: Option<String>,
}

#[derive(Debug)]
pub struct TagStop {
    pub offset: f32,
    pub color: Color,
    pub opacity: f32,
}

impl Tag for TagLinearGradient {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
impl Tag for TagRadialGradient {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
impl ParseNode for TagLinearGradient {
    fn parse_node(node: &Node) -> Result<TagLinearGradient, Error> {
        let x1 = node.attribute("x1").map(Length::from_str).transpose()?;
        let y1 = node.attribute("y1").map(Length::from_str).transpose()?;
        let x2 = node.attribute("x2").map(Length::from_str).transpose()?;
        let y2 = node.attribute("y2").map(Length::from_str).transpose()?;
        let gradient_transform = node.attribute("gradientTransform").map(transform_list).transpose()?;
        let id = node.attribute("id").map(|s| s.to_owned());
        let href = href(node);
    
        let mut stops = Vec::new();
        for elem in node.children().filter(|n| n.is_element()) {
            match elem.tag_name().name() {
                "stop" => stops.push(TagStop::parse_node(&elem)?),
                _ => {}
            }
        }
    
        Ok(TagLinearGradient {
            from: (x1, y1),
            to: (x2, y2),
            gradient_transform,
            stops,
            id,
            href
        })
    }
}
impl ParseNode for TagRadialGradient {
    fn parse_node(node: &Node) -> Result<TagRadialGradient, Error> {
        let cx = node.attribute("cx").map(Length::from_str).transpose()?;
        let cy = node.attribute("cy").map(Length::from_str).transpose()?;
        let r = node.attribute("r").map(Length::from_str).transpose()?;
        let fx = node.attribute("x2").map(Length::from_str).transpose()?;
        let fy = node.attribute("y2").map(Length::from_str).transpose()?;
        let gradient_transform = node.attribute("gradientTransform").map(transform_list).transpose()?;
        let id = node.attribute("id").map(|s| s.to_owned());
        let href = href(node);
    
        let mut stops = Vec::new();
        for elem in node.children().filter(|n| n.is_element()) {
            match elem.tag_name().name() {
                "stop" => stops.push(TagStop::parse_node(&elem)?),
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
            href,
        })
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

    pub fn color_u(&self, opacity: f32) -> ColorU {
        let Color { red, green, blue } = self.color;
        let alpha = (opacity * self.opacity * 255.) as u8;
        ColorU::new(red, green, blue, alpha)
    }
}
impl ParseNode for TagStop {
    fn parse_node(node: &Node) -> Result<TagStop, Error> {
        let mut stop = TagStop::new();

        for attr in node.attributes() {
            stop.apply(attr.name(), attr.value());
        }

        Ok(stop)
    }
}

fn number_or_percent(s: &str) -> Result<f32, Error> {
    match Length::from_str(s)? {
        Length { num, unit: LengthUnit::None } => Ok(num as f32),
        Length { num, unit: LengthUnit::Percent } => Ok(0.01 * num as f32),
        _ => Err(Error::InvalidAttributeValue("number or percent".into()))
    }
}

