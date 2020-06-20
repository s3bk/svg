use crate::prelude::*;
use crate::{parse_node_list, link};
use libflate::gzip::Decoder;

use std::sync::Arc;
use roxmltree::{Document};

#[derive(Debug)]
pub struct Svg {
    items: Vec<Arc<Item>>,
    view_box: Option<Rect>,
    pub named_items: ItemCollection,
}
impl Svg {
    pub fn compose(&self) -> Scene {
        let mut scene = Scene::new();
        let ctx = DrawContext::new(self);
        let options = DrawOptions::new(&ctx);
        if let Some(ref r) = self.view_box {
            scene.set_view_box(options.transform * r.as_rectf());
        }
        self.compose_to(&mut scene, options);
        scene
    }
    pub fn compose_to(&self, scene: &mut Scene, options: DrawOptions) {
        for item in &self.items {
            item.compose_to(scene, &options);
        }
    }
    pub fn compose_node(&self, id: &str) -> Option<Scene> {
        self.named_items.get(id).map(|item| {
            let mut scene = Scene::new();
            let ctx = DrawContext::new(self);
            let options = DrawOptions::new(&ctx);
            item.compose_to(&mut scene, &options);
            scene
        })
    }
    pub fn parse<'a>(doc: &'a Document) -> Result<Svg, Error> {
        let root = doc.root_element();
        assert!(root.has_tag_name("svg"));
        let view_box = root.attribute("viewBox").map(Rect::parse).transpose()?;
        let width = root.attribute("width").map(length).transpose()?;
        let height = root.attribute("height").map(length).transpose()?;
    
        let view_box = match (view_box, width, height) {
            (Some(r), _, _) => Some(r),
            (None, Some(w), Some(h)) => Some(Rect::from_size(w, h)),
            _ => None
        };

        let items = parse_node_list(root.children())?;
    
        let mut named_items = ItemCollection::new();
        for item in &items {
            link(&mut named_items, item);
        }
    
        Ok(Svg { items, view_box, named_items })
    }
    pub fn from_str(text: &str) -> Result<Svg, Error> {
        timed!("parse xml", {
            let doc = Document::parse(text)?;
        });
        timed!("build svg", {
            Svg::parse(&doc)
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