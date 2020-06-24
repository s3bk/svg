use crate::prelude::*;
use crate::{parse_node, parse_node_list, link};
use libflate::gzip::Decoder;

use std::sync::Arc;
use roxmltree::{Document};

#[derive(Debug)]
pub struct TagSvg {
    pub (crate) id: Option<String>,
    pub (crate) items: Vec<Arc<Item>>,
    pub (crate) view_box: Option<Rect>,
    pub attrs: Attrs,
}

#[derive(Debug)]
pub struct Svg {
    pub named_items: ItemCollection,
    root: Arc<Item>,
}
impl Tag for TagSvg {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        self.view_box.as_ref().map(|r| options.resolve_rect(r))
        .or_else(|| max_bounds(self.items.iter().flat_map(|item| item.bounds(&options))))
    }
    fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(&self.attrs);
        for item in self.items.iter() {
            item.compose_to(scene, &options);
        }
    }
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
    fn children(&self) -> &[Arc<Item>] {
        &*self.items
    }
}

impl TagSvg {
    pub fn parse(node: &Node) -> Result<TagSvg, Error> {
        let view_box = node.attribute("viewBox").map(Rect::parse).transpose()?;
        let width = node.attribute("width").map(length).transpose()?;
        let height = node.attribute("height").map(length).transpose()?;
        let id = node.attribute("id").map(|s| s.into());
        let attrs = Attrs::parse(node)?;
    
        let view_box = match (view_box, width, height) {
            (Some(r), _, _) => Some(r),
            (None, Some(w), Some(h)) => Some(Rect::from_size(w, h)),
            _ => None
        };

        let items = parse_node_list(node.children())?;
    
        Ok(TagSvg { items, view_box, id, attrs })
    }
}

impl Svg {
    pub fn compose(&self) -> Scene {
        self.compose_with_transform(Transform2F::default())
    }

    pub fn compose_with_transform(&self, transform: Transform2F) -> Scene {
        let ctx = DrawContext::new(self);
        let mut options = DrawOptions::new(&ctx);
        options.transform = transform;
        self.compose_with_options(&options)
    }

    pub fn compose_with_options(&self, options: &DrawOptions) -> Scene {
        let mut scene = Scene::new();
        
        if let Item::Svg(TagSvg { view_box: Some(r), .. }) = &*self.root {
            scene.set_view_box(options.transform * options.resolve_rect(r));
        }
        self.root.compose_to(&mut scene, options);
        scene
    }

    /// get the viewbox (computed if missing)
    pub fn view_box(&self) -> Option<RectF> {
        let ctx = DrawContext::new(self);
        let options = DrawOptions::new(&ctx);
        
        if let Item::Svg(TagSvg { view_box: Some(r), .. }) = &*self.root {
            return Some(options.resolve_rect(r));
        } else {
            self.root.bounds(&options)
        }
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