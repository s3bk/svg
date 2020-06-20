use crate::prelude::*;
use crate::{parse_node, parse_node_list, link};
use libflate::gzip::Decoder;

use std::sync::Arc;
use roxmltree::{Document};

#[derive(Debug)]
pub struct TagSvg {
    pub id: Option<String>,
    items: Vec<Arc<Item>>,
    view_box: Option<Rect>,
}

#[derive(Debug)]
pub struct Svg {
    pub named_items: ItemCollection,
    root: Arc<Item>,
}

impl TagSvg {
    pub fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        for item in &self.items {
            item.compose_to(scene, &options);
        }
    }
    pub fn parse(node: &Node) -> Result<TagSvg, Error> {
        let view_box = node.attribute("viewBox").map(Rect::parse).transpose()?;
        let width = node.attribute("width").map(length).transpose()?;
        let height = node.attribute("height").map(length).transpose()?;
        let id = node.attribute("id").map(|s| s.into());
    
        let view_box = match (view_box, width, height) {
            (Some(r), _, _) => Some(r),
            (None, Some(w), Some(h)) => Some(Rect::from_size(w, h)),
            _ => None
        };

        let items = parse_node_list(node.children())?;
    
        Ok(TagSvg { items, view_box, id })
    }
}

impl Svg {
    pub fn compose(&self) -> Scene {
        let mut scene = Scene::new();
        let ctx = DrawContext::new(self);
        let options = DrawOptions::new(&ctx);

        if let Item::Svg(TagSvg { view_box: Some(r), .. }) = &*self.root {
            scene.set_view_box(options.transform * r.as_rectf());
        }
        self.root.compose_to(&mut scene, &options);
        scene
    }
    pub fn get_item(&self, id: &str) -> Option<&Arc<Item>> {
        self.named_items.get(id)
    }
    pub fn from_str(text: &str) -> Result<Svg, Error> {
        timed!("parse xml", {
            let doc = Document::parse(text)?;
        });
        timed!("build svg", {
            let root = parse_node(&doc.root_element());
        });
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
            timed!("inflate", {
                use std::io::Read;
                let mut decoder = Decoder::new(data).map_err(Error::Gzip)?;
                let mut decoded_data = Vec::new();
                decoder.read_to_end(&mut decoded_data).map_err(Error::Gzip)?;
            });
            let text = std::str::from_utf8(&decoded_data)?;
            Self::from_str(text)
        } else {
            let text = std::str::from_utf8(data)?;
            Self::from_str(text)
        }
    }
}