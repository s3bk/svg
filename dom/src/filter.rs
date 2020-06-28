use crate::prelude::*;
use pathfinder_renderer::{
    scene::{RenderTarget, DrawPath},
    paint::Paint,
};
use pathfinder_content::{
    pattern::{Pattern},
    effects::{PatternFilter, BlurDirection},
    outline::Outline,
    render_target::{RenderTargetId},
};
use pathfinder_geometry::rect::RectI;

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
    GaussianBlur(FeGaussianBlur)
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
