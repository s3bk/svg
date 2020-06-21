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
impl TagFilter {
    pub fn apply(&self, scene: &mut Scene, options: &DrawOptions, bounds: RectF, f: impl FnOnce(&mut Scene, &DrawOptions)) {
        if let Some(first) = self.filters.first() {
            let mut options2 = options.clone();
            let info = first.pre(scene, bounds, &mut options2);
            f(scene, &options2);
            info.post(scene, options);
        } else {
            f(scene, options);
        }
    }
    pub fn parse<'a, 'i: 'a>(node: &Node<'a, 'i>) -> Result<TagFilter, Error> {
        let mut filters = Vec::with_capacity(1);
        for elem in node.children().filter(|n| n.is_element()) {
            let filter = match elem.tag_name().name() {
                "feGaussianBlur" => Filter::GaussianBlur(FeGaussianBlur::parse(&elem)?),
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
enum FilterState {
    GaussianBlur(GaussianBlurInfo)
}
impl Filter {
    fn pre(&self, scene: &mut Scene, outline_bounds: RectF, options: &mut DrawOptions) -> FilterState {
        match *self {
            Filter::GaussianBlur(ref f) => FilterState::GaussianBlur(f.pre(scene, outline_bounds, options)),
        }
    }
}
impl FilterState {
    fn post(self, scene: &mut Scene, options: &DrawOptions) {
        match self {
            FilterState::GaussianBlur(info) => FeGaussianBlur::post(info, scene, options),
        }
    }
}

#[derive(Debug)]
pub struct FeGaussianBlur {
    std_deviation: f32
}
struct GaussianBlurInfo {
    sigma: Vector2F,
    bounds: RectI,
    render_target_id_y: RenderTargetId,
    render_target_id_x: RenderTargetId,
}
impl FeGaussianBlur {
    fn parse<'a, 'i: 'a>(node: &Node<'a, 'i>) -> Result<FeGaussianBlur, Error> {
        let std_deviation = node.attribute("stdDeviation").map(f32::from_str).transpose()?.unwrap_or_default();
        Ok(FeGaussianBlur { std_deviation })
    }
    fn pre(&self, scene: &mut Scene, outline_bounds: RectF, options: &mut DrawOptions) -> GaussianBlurInfo {
        let sigma = options.transform.extract_scale() * self.std_deviation;
        let bounds = outline_bounds.dilate(sigma * 3.0).round_out().to_i32();
        dbg!(bounds);

        let render_target_y = RenderTarget::new(bounds.size(), String::new());
        let render_target_id_y = scene.push_render_target(render_target_y);
        let render_target_x = RenderTarget::new(bounds.size(), String::new());
        let render_target_id_x = scene.push_render_target(render_target_x);

        options.transform = Transform2F::from_translation(-bounds.origin().to_f32()) * options.transform;
        dbg!(options.transform);

        GaussianBlurInfo {
            render_target_id_x,
            render_target_id_y,
            sigma,
            bounds
        }
    }
    fn post(info: GaussianBlurInfo, scene: &mut Scene, options: &DrawOptions) {
        let GaussianBlurInfo {
            render_target_id_x,
            render_target_id_y,
            sigma,
            bounds
        } = info;

        let mut paint_x = Pattern::from_render_target(render_target_id_x, bounds.size());
        let mut paint_y = Pattern::from_render_target(render_target_id_y, bounds.size());
        paint_y.apply_transform(Transform2F::from_translation(bounds.origin().to_f32()));

        paint_x.set_filter(Some(PatternFilter::Blur { direction: BlurDirection::X, sigma: sigma.x() }));
        paint_y.set_filter(Some(PatternFilter::Blur { direction: BlurDirection::Y, sigma: sigma.y() }));

        let paint_id_x = scene.push_paint(&Paint::from_pattern(paint_x));
        let paint_id_y = scene.push_paint(&Paint::from_pattern(paint_y));
        //let clip_path = options.clip_path_id(scene);

        // TODO(pcwalton): Apply clip as necessary.
        let outline_x = Outline::from_rect(RectF::new(vec2f(0.0, 0.0), bounds.size().to_f32()));
        let path_x = DrawPath::new(outline_x, paint_id_x);
        let outline_y = Outline::from_rect(bounds.to_f32());
        let mut path_y = DrawPath::new(outline_y, paint_id_y);
        //path_y.set_clip_path(clip_path);

        scene.pop_render_target();
        scene.push_path(path_x);
        scene.pop_render_target();
        scene.push_path(path_y);
    }
}
