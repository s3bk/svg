use crate::prelude::*;
use crate::{parse_node, parse_node_list, link};
use libflate::gzip::Decoder;

use std::sync::Arc;
use roxmltree::{Document};

#[derive(Debug)]
pub struct TagSvg {
    pub id: Option<String>,
    pub items: Vec<Arc<Item>>,
    pub view_box: Option<Rect>,
    pub width: Option<LengthX>,
    pub height: Option<LengthY>,
    pub attrs: Attrs,
}

#[derive(Debug, Clone)]
pub struct Svg {
    pub named_items: ItemCollection,
    pub root: Arc<Item>,
}
impl Tag for TagSvg {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
    fn children(&self) -> &[Arc<Item>] {
        &*self.items
    }
}

impl ParseNode for TagSvg {
    fn parse_node(node: &Node) -> Result<TagSvg, Error> {
        let view_box = node.attribute("viewBox").map(Rect::parse).transpose()?;
        let width = node.attribute("width").map(LengthX::parse).transpose()?;
        let height = node.attribute("height").map(LengthY::parse).transpose()?;
        let id = node.attribute("id").map(|s| s.into());
        let attrs = Attrs::parse(node)?;

        let items = parse_node_list(node.children())?;
    
        Ok(TagSvg { items, view_box, id, attrs, width, height })
    }
}

impl Svg {
    pub fn get_item(&self, id: &str) -> Option<&Arc<Item>> {
        self.named_items.get(id)
    }
    pub fn from_str(text: &str) -> Result<Svg, Error> {
        let doc = Document::parse(text)?;
        let root = parse_node(&doc.root_element(), true, true);
        let root_item = Arc::new(root?.ok_or(Error::NotSvg)?);

        let mut named_items = ItemCollection::new();
        link(&mut named_items, &root_item);

        Ok(Svg {
            root: root_item,
            named_items,
        })
    }
    pub fn from_data(data: &[u8]) -> Result<Svg, Error> {
        if data.starts_with(&[0x1f, 0x8b]) {
            use std::io::Read;
            let mut decoder = Decoder::new(data).map_err(Error::Gzip)?;
            let mut decoded_data = Vec::new();
            decoder.read_to_end(&mut decoded_data).map_err(Error::Gzip)?;
            let text = std::str::from_utf8(&decoded_data)?;
            Self::from_str(text)
        } else {
            let text = std::str::from_utf8(data)?;
            Self::from_str(text)
        }
    }
}