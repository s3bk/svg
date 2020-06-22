use crate::prelude::*;
use crate::{parse_node_list, TagSvg};
use std::sync::Arc;

#[derive(Debug)]
pub struct TagG {
    items: Vec<Arc<Item>>,
    attrs: Attrs,
    id: Option<String>,
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
        compose_items(scene, &self.items, &self.attrs, options);
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
fn compose_items(scene: &mut Scene, items: &[Arc<Item>], attrs: &Attrs, options: &DrawOptions) {
    if !attrs.display {
        return;
    }

    let options = options.apply(attrs);

    if let Some(ref filter_id) = attrs.filter {
        let bounds = get_or_return!(max_bounds(items.iter().flat_map(|item| item.bounds(&options))));

        match options.ctx.resolve(&filter_id).map(|i| &**i) {
            Some(Item::Filter(filter)) => {
                filter.apply(scene, &options, bounds, |scene, options| {
                    for item in items {
                        item.as_ref().compose_to(scene, options);
                    }
                });
                return;
            },
            r => println!("expected filter, got {:?}", r)
        }
    }

    for item in items.iter() {
        item.compose_to(scene, &options);
    }
}

#[derive(Debug)]
pub struct TagSymbol {
    items: Vec<Arc<Item>>,
    attrs: Attrs,
    id: Option<String>,
    view_box: Option<Rect>,
}
impl Tag for TagSymbol {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
    fn children(&self) -> &[Arc<Item>] {
        &*self.items
    }
}
impl TagSymbol {
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagSymbol, Error> {
        let attrs = Attrs::parse(node)?;
        let items = parse_node_list(node.children())?;
        let id = node.attribute("id").map(|s| s.into());
        let view_box = node.attribute("viewBox").map(Rect::parse).transpose()?;

        Ok(TagSymbol { items, attrs, id,view_box })
    }
}

#[derive(Debug)]
pub struct TagUse {
    attrs: Attrs,
    x: Option<Length>,
    y: Option<Length>,
    width: Option<Length>,
    height: Option<Length>,
    href: Option<String>,
    id: Option<String>,
}

impl TagUse {
    pub fn parse<'i, 'a: 'i>(node: &Node<'i, 'a>) -> Result<TagUse, Error> {
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
    fn content_transform<'a, 'b: 'a>(&'b self, options: &DrawOptions<'a, 'b>, item: &Item) -> DrawOptions<'a, 'b> {
        let mut options = options.apply(&self.attrs);
        let pos = options.resolve_point((self.x.unwrap_or_default(), self.y.unwrap_or_default()));
        options.transform(Transform2F::from_translation(pos));
        match *item {
            Item::Symbol(TagSymbol { view_box: Some(ref view_box), .. }) |
            Item::Svg(TagSvg { view_box: Some(ref view_box), .. }) => {
                let size = (
                    self.width.unwrap_or(view_box.width),
                    self.height.unwrap_or(view_box.height)
                );
                options.apply_viewbox(size, view_box);
            }
            _ => {}
        }
        options
    }
}
impl Tag for TagUse {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if !self.attrs.display {
            return None;
        }
        let item = &**options.ctx.resolve_href(self.href.as_ref()?)?;
        let options = self.content_transform(options, item);
        item.bounds(&options)
    }
    fn compose_to(&self, scene: &mut Scene, options: &DrawOptions) {
        if !self.attrs.display {
            return;
        }
        let href = get_ref_or_return!(self.href, "<use> without href");
        let item = get_or_return!(options.ctx.resolve_href(href), "can't resolve <use href={:?}>", href);
        let options = self.content_transform(&options, item);
        debug!("item: {:?}", *item);
        match **item {
            Item::Symbol(TagSymbol { ref items, ref attrs, .. }) |
            Item::Svg(TagSvg { ref items, ref attrs, .. }) |
            Item::G(TagG { ref items, ref attrs, ..}) => {
                compose_items(scene, &items, attrs, &options);
            }
            ref item => {
                item.compose_to(scene, &options);
            }
        }
    }
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
}
