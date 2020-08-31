use crate::prelude::*;
use pathfinder_simd::default::F32x4;
use svgtypes::NumberListParser;

#[derive(Debug)]
pub struct TagFilter {
    pub filters: Vec<Filter>,
    pub id: Option<String>,
}
impl Tag for TagFilter {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
impl ParseNode for TagFilter {
    fn parse_node(node: &Node) -> Result<TagFilter, Error> {
        let mut filters = Vec::with_capacity(1);
        for elem in node.children().filter(|n| n.is_element()) {
            let filter = match elem.tag_name().name() {
                "feGaussianBlur" => Filter::GaussianBlur(FeGaussianBlur::parse_node(&elem)?),
                "feColorMatrix" => Filter::ColorMatrix(FeColorMatrix::parse_node(&elem)?),
                name => {
                    print!("unimplemented filter: {}", name);
                    continue;
                }
            };
            filters.push(filter);
        }
        
        let id = node.attribute("id").map(|s| s.to_owned());

        Ok(TagFilter { id, filters })
    }
}

#[derive(Debug)]
pub enum Filter {
    GaussianBlur(FeGaussianBlur),
    ColorMatrix(FeColorMatrix),
}

#[derive(Debug)]
pub struct FeGaussianBlur {
    pub std_deviation: f32
}
impl ParseNode for FeGaussianBlur {
    fn parse_node(node: &Node) -> Result<FeGaussianBlur, Error> {
        let std_deviation = node.attribute("stdDeviation").map(f32::from_str).transpose()?.unwrap_or_default();
        Ok(FeGaussianBlur { std_deviation })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FeColorMatrix {
    Matrix([F32x4; 5]),
    HueRotate(f32),
    Saturate(f32),
    LuminanceToAlpha,
}
impl ParseNode for FeColorMatrix {
    fn parse_node(node: &Node) -> Result<FeColorMatrix, Error> {
        let typ = node.attribute("type").unwrap_or("matrix");
        match typ {
            "matrix" => {
                let values = node.attribute("values").ok_or_else(|| Error::MissingAttribute("values".into()))?;
                let values: Vec<f32> = NumberListParser::from(values).map(|r| r.map(|v| v as f32)).collect::<Result<Vec<_>, _>>()?;
                if values.len() != 20 {
                    return Err(Error::InvalidAttributeValue(format!("expected 20 values, got {}", values.len())));
                }
                Ok(FeColorMatrix::Matrix([
                    F32x4::new(values[0], values[5], values[10], values[15]),
                    F32x4::new(values[1], values[6], values[11], values[16]),
                    F32x4::new(values[2], values[7], values[12], values[17]),
                    F32x4::new(values[3], values[8], values[13], values[18]),
                    F32x4::new(values[4], values[9], values[14], values[19]),
                ]))
            }
            "saturate"=> {
                let values = node.attribute("values").ok_or_else(|| Error::MissingAttribute("values".into()))?;
                let value: f32 = values.parse()?;
                Ok(FeColorMatrix::Saturate(value))
            }
            "hueRotate" => {
                let values = node.attribute("values").ok_or_else(|| Error::MissingAttribute("values".into()))?;
                let deg: f32 = values.parse()?;
                Ok(FeColorMatrix::HueRotate(deg2rad(deg)))
            },
            "luminanceToAlpha" => Ok(FeColorMatrix::LuminanceToAlpha),
            _ => Err(Error::InvalidAttributeValue("type".into()))
        }
    }
}
