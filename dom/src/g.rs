use crate::prelude::*;
use crate::{parse_node_list, TagSvg};
use std::sync::Arc;

#[derive(Debug)]
pub struct TagG {
    pub items: Vec<Arc<Item>>,
    pub attrs: Attrs,
    pub id: Option<String>,
}
impl Tag for TagG {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
    fn children(&self) -> &[Arc<Item>] {
        &*self.items
    }
}
impl ParseNode for TagG {
    fn parse_node(node: &Node) -> Result<TagG, Error> {
        let attrs = Attrs::parse(node)?;
        let items = parse_node_list(node.children())?;
        let id = node.attribute("id").map(|s| s.into());
        Ok(TagG { items, attrs, id })
    }
}

#[derive(Debug)]
pub struct TagSymbol {
    pub items: Vec<Arc<Item>>,
    pub attrs: Attrs,
    pub id: Option<String>,
    pub view_box: Option<Rect>,
}
impl Tag for TagSymbol {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
    fn children(&self) -> &[Arc<Item>] {
        &*self.items
    }
}
impl ParseNode for TagSymbol {
    fn parse_node(node: &Node) -> Result<TagSymbol, Error> {
        let attrs = Attrs::parse(node)?;
        let items = parse_node_list(node.children())?;
        let id = node.attribute("id").map(|s| s.into());
        let view_box = node.attribute("viewBox").map(Rect::parse).transpose()?;

        Ok(TagSymbol { items, attrs, id,view_box })
    }
}

#[derive(Debug)]
pub struct TagUse {
    pub attrs: Attrs,
    pub x: Option<Length>,
    pub y: Option<Length>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub href: Option<String>,
    pub id: Option<String>,
}

impl ParseNode for TagUse {
    fn parse_node(node: &Node) -> Result<TagUse, Error> {
        let x = node.attribute("x").map(length).transpose()?;
        let y = node.attribute("y").map(length).transpose()?;
        let width = node.attribute("width").map(length).transpose()?;
        let height = node.attribute("height").map(length).transpose()?;
        let href = href(node);
        let attrs = Attrs::parse(node)?;
        let id = node.attribute("id").map(|s| s.into());

        Ok(TagUse {
            x, y, width, height, attrs, href, id,
        })
    }
}
impl Tag for TagUse {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
