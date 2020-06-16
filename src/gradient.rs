use crate::prelude::*;
use pathfinder_content::gradient::{Gradient, GradientGeometry, ColorStop};
use pathfinder_color::{ColorU, ColorF};
use pathfinder_geometry::line_segment::LineSegment2F;
use pathfinder_simd::default::F32x2;
use svgtypes::Color;

#[derive(Debug)]
pub struct TagLinearGradient {
    pub from: (Length, Length),
    pub to: (Length, Length),
    pub gradient_transform: Transform2F,
    pub stops: Vec<TagStop>,
    pub id: Option<String>,
}

#[derive(Debug)]
pub struct TagRadialGradient {
    pub center: (Length, Length),
    pub focus: (Length, Length),
    pub radius: Length,
    pub gradient_transform: Transform2F,
    pub stops: Vec<TagStop>,
    pub id: Option<String>,
}

#[derive(Debug)]
pub struct TagStop {
    pub offset: f32,
    pub color: ColorF,
}

impl TagLinearGradient {
    pub fn parse<'a, 'i: 'a>(node: &Node<'a, 'i>) -> Result<TagLinearGradient, Error<'a>> {
        let x1 = node.attribute("x1").map(Length::from_str).transpose()?.unwrap_or(Length::new(0.0, LengthUnit::Percent));
        let y1 = node.attribute("y1").map(Length::from_str).transpose()?.unwrap_or(Length::new(0.0, LengthUnit::Percent));
        let x2 = node.attribute("x2").map(Length::from_str).transpose()?.unwrap_or(Length::new(100.0, LengthUnit::Percent));
        let y2 = node.attribute("y2").map(Length::from_str).transpose()?.unwrap_or(Length::new(0.0, LengthUnit::Percent));
        let gradient_transform = node.attribute("gradientTransform").map(transform_list).transpose()?.unwrap_or_default();
        let id = node.attribute("id").map(|s| s.to_owned());
    
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
            id
        })
    }

    pub fn build(&self, options: &DrawOptions, opacity: f32) -> Gradient {
        let mut gradient = Gradient::linear_from_points(
            options.resolve_point(self.from),
            options.resolve_point(self.to)
        );
        for stop in &self.stops {
            let mut color = stop.color;
            color.set_a(color.a() * opacity);
            gradient.add_color_stop(color.to_u8(), stop.offset);
        }

        gradient.apply_transform(options.transform * self.gradient_transform);
        gradient
    }
}

impl TagRadialGradient {
    pub fn parse<'a, 'i: 'a>(node: &Node<'a, 'i>) -> Result<TagRadialGradient, Error<'a>> {
        let cx = node.attribute("cx").map(Length::from_str).transpose()?.unwrap_or(Length::new(50.0, LengthUnit::Percent));
        let cy = node.attribute("cy").map(Length::from_str).transpose()?.unwrap_or(Length::new(50.0, LengthUnit::Percent));
        let r = node.attribute("r").map(Length::from_str).transpose()?.unwrap_or(Length::new(50.0, LengthUnit::Percent));
        let fx = node.attribute("x2").map(Length::from_str).transpose()?.unwrap_or(cx);
        let fy = node.attribute("y2").map(Length::from_str).transpose()?.unwrap_or(cy);
        let gradient_transform = node.attribute("gradientTransform").map(transform_list).transpose()?.unwrap_or_default();
        let id = node.attribute("id").map(|s| s.to_owned());
    
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
        })
    }

    pub fn build(&self, options: &DrawOptions, opacity: f32) -> Gradient {
        let mut gradient = Gradient::radial(
            LineSegment2F::new(
                options.resolve_point(self.focus),
                options.resolve_point(self.center)
            ),
            F32x2::new(0.0, options.resolve_length(self.radius))
        );
        for stop in &self.stops {
            let mut color = stop.color;
            color.set_a(color.a() * opacity);
            gradient.add_color_stop(color.to_u8(), stop.offset);
        }

        gradient.apply_transform(options.transform * self.gradient_transform);
        gradient
    }
}

fn number_or_percent(s: &str) -> Result<f32, Error> {
    match Length::from_str(s)? {
        Length { num, unit: LengthUnit::None } => Ok(num as f32),
        Length { num, unit: LengthUnit::Percent } => Ok(0.01 * num as f32),
        _ => Err(Error::InvalidAttributeValue("number or percent"))
    }
}


impl TagStop {
    pub fn parse<'a, 'i: 'a>(node: &Node<'a, 'i>) -> Result<TagStop, Error<'a>> {
        let offset = number_or_percent(node.attribute("offset").ok_or(Error::MissingAttribute("offset"))?)?;
        let offset = offset.max(0.0).min(1.0);
        let opacity = node.attribute("stop-opacity").map(opacity).transpose()?.unwrap_or(1.0);

        let color = match node.attribute("stop-color") {
            Some(color) => {
                let Color { red, green, blue } = Color::from_str(color)?;
                let mut color = ColorU::new(red, green, blue, 0).to_f32();
                color.set_a(opacity);
                color
            },
            None => ColorF::new(0., 0., 0., opacity)
        };

        Ok(TagStop {
            offset, color
        })
    }
}

