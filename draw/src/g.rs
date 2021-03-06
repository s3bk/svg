use crate::prelude::*;
use std::sync::Arc;
use crate::filter::apply_filter;

impl DrawItem for TagG {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if !self.attrs.display {
            return None;
        }

        let options = options.apply(&self.attrs);
        max_bounds(self.items.iter().flat_map(|item| item.bounds(&options)))
    }
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        draw_items(scene, &self.items, &self.attrs, options);
    }
}

fn draw_items(scene: &mut Scene, items: &[Arc<Item>], attrs: &Attrs, options: &DrawOptions) {
    if !attrs.display {
        return;
    }

    let options = options.apply(attrs);

    if let Some(Iri(ref filter_id)) = attrs.filter {
        let bounds = get_or_return!(max_bounds(items.iter().flat_map(|item| item.bounds(&options))));

        match options.ctx.resolve(&filter_id).map(|i| &**i) {
            Some(Item::Filter(filter)) => {
                apply_filter(filter, scene, &options, bounds, |scene, options| {
                    for item in items {
                        item.as_ref().draw_to(scene, options);
                    }
                });
                return;
            },
            r => println!("expected filter for {:?}, got {:?}", filter_id, r)
        }
    }

    for item in items.iter() {
        item.draw_to(scene, &options);
    }
}
fn content_transform<'a>(tag: &TagUse, options: &DrawOptions<'a>, item: &Item) -> DrawOptions<'a> {
    let mut options = options.apply(&tag.attrs);
    let pos = tag.pos.resolve(&options);
    options.transform(Transform2F::from_translation(pos));
    match *item {
        Item::Symbol(TagSymbol { view_box: Some(ref view_box), .. }) |
        Item::Svg(TagSvg { view_box: Some(ref view_box), .. }) => {
            options.apply_viewbox(tag.width, tag.height, view_box);
        }
        _ => {}
    }
    options
}

impl DrawItem for TagUse {
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        if !self.attrs.display {
            return None;
        }
        let item = &**options.ctx.resolve_href(self.href.as_ref()?)?;
        let options = content_transform(self, options, item);
        item.bounds(&options)
    }
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        if !self.attrs.display {
            return;
        }
        let href = get_ref_or_return!(self.href, "<use> without href");
        let item = get_or_return!(options.ctx.resolve_href(href), "can't resolve <use href={:?}>", href);
        let options = content_transform(&self, options, item);
        debug!("item: {:?}", *item);
        match **item {
            Item::Symbol(TagSymbol { ref items, ref attrs, .. }) |
            Item::Svg(TagSvg { ref items, ref attrs, .. }) |
            Item::G(TagG { ref items, ref attrs, ..}) => {
                draw_items(scene, &items, attrs, &options);
            }
            ref item => {
                item.draw_to(scene, &options);
            }
        }
    }
}
