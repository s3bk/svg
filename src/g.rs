use crate::prelude::*;
use crate::{parse_node_list};
use std::sync::Arc;

#[derive(Debug)]
pub struct TagG {
    pub items: Vec<Arc<Item>>,
    attrs: Attrs,
    pub id: Option<String>,
}
impl Tag for TagG {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if !self.attrs.display {
            return None;
        }

        let options = options.apply(&self.attrs);
        max_bounds(self.items.iter().flat_map(|item| item.bounds(&options)))
    }
    fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        if !self.attrs.display {
            return;
        }

        let options = options.apply(&self.attrs);

        if let Some(ref filter_id) = self.attrs.filter {
            let bounds = match max_bounds(self.items.iter().flat_map(|item| item.bounds(&options))) {
                Some(b) => b,
                None => {
                    println!("no bounds for {:?}", self);
                    return;
                }
            };
            match options.ctx.resolve(&filter_id).map(|i| &**i) {
                Some(Item::Filter(filter)) => {
                    filter.apply(scene, &options, bounds, |scene, options| {
                        for item in &self.items {
                            item.compose_to(scene, options);
                        }
                    });
                    return;
                },
                r => println!("expected filter, got {:?}", r)
            }
        }

        for item in &self.items {
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
impl TagG {
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagG, Error> {
        let attrs = Attrs::parse(node)?;
        let items = parse_node_list(node.children())?;
        let id = node.attribute("id").map(|s| s.into());
        Ok(TagG { items, attrs, id })
    }
}