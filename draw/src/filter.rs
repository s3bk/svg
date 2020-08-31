use crate::prelude::*;
use pathfinder_renderer::{
    scene::{RenderTarget, DrawPath},
    paint::Paint,
};
use pathfinder_content::{
    pattern::{Pattern},
    effects::{PatternFilter, BlurDirection, BlendMode},
    outline::Outline,
    render_target::{RenderTargetId},
};
use pathfinder_geometry::rect::RectI;
use pathfinder_color::matrix::ColorMatrix;

pub fn apply_filter(filter: &TagFilter, scene: &mut Scene, options: &DrawOptions, bounds: RectF, f: impl FnOnce(&mut Scene, &DrawOptions)) {
    if let Some(first) = filter.filters.first() {
        let mut options2 = options.clone();
        let info = FilterState::pre(first, scene, bounds, &mut options2);
        f(scene, &options2);
        info.post(scene, options);
    } else {
        f(scene, options);
    }
}

enum FilterState {
    GaussianBlur(GaussianBlurInfo),
    ColorMatrix(ColorMatrixInfo),
}
impl FilterState {
    fn pre(filter: &Filter, scene: &mut Scene, outline_bounds: RectF, options: &mut DrawOptions) -> FilterState {
        match *filter {
            Filter::GaussianBlur(ref f) => {
                let sigma = options.transform.extract_scale() * f.std_deviation;
                let bounds = outline_bounds.dilate(sigma * 3.0).round_out().to_i32();
        
                let render_target_y = RenderTarget::new(bounds.size(), String::new());
                let render_target_id_y = scene.push_render_target(render_target_y);
                let render_target_x = RenderTarget::new(bounds.size(), String::new());
                let render_target_id_x = scene.push_render_target(render_target_x);
        
                options.transform = Transform2F::from_translation(-bounds.origin().to_f32()) * options.transform;
        
                FilterState::GaussianBlur(GaussianBlurInfo {
                    render_target_id_x,
                    render_target_id_y,
                    sigma,
                    bounds
                })
            }
            Filter::ColorMatrix(filter) => {
                let bounds = outline_bounds.round_out().to_i32();
                let render_target = RenderTarget::new(bounds.size(), String::new());
                let render_target_id = scene.push_render_target(render_target);
                options.transform = Transform2F::from_translation(-bounds.origin().to_f32()) * options.transform;

                FilterState::ColorMatrix(ColorMatrixInfo {
                    render_target_id,
                    bounds,
                    filter, 
                })
            }
        }
    }
    fn post(self, scene: &mut Scene, options: &DrawOptions) {
        match self {
            FilterState::GaussianBlur(info) => {
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
                let path_y = DrawPath::new(outline_y, paint_id_y);
                //path_y.set_clip_path(clip_path);
        
                scene.pop_render_target();
                scene.push_draw_path(path_x);
                scene.pop_render_target();
                scene.push_draw_path(path_y);
            }
            FilterState::ColorMatrix(info) => {
                let ColorMatrixInfo {
                    render_target_id,
                    bounds,
                    filter
                } = info;

                let mut paint = Pattern::from_render_target(render_target_id, bounds.size());
                let (matrix, blend_mode) = match filter {
                    FeColorMatrix::Matrix(columns) => (ColorMatrix(columns), BlendMode::SrcOver),
                    FeColorMatrix::Saturate(saturation) => (ColorMatrix::saturate(saturation), BlendMode::SrcOver),
                    FeColorMatrix::HueRotate(radians) => (ColorMatrix::hue_rotate(radians), BlendMode::SrcOver),
                    FeColorMatrix::LuminanceToAlpha => (ColorMatrix::luminance_to_alpha(), BlendMode::DestIn),
                };
                paint.set_filter(Some(PatternFilter::ColorMatrix(matrix)));
                paint.apply_transform(Transform2F::from_translation(bounds.origin().to_f32()));

                let paint_id = scene.push_paint(&Paint::from_pattern(paint));
                let outline = Outline::from_rect(bounds.to_f32());
                let mut path = DrawPath::new(outline, paint_id);
                path.set_blend_mode(blend_mode);

                scene.pop_render_target();
                scene.push_draw_path(path);
            }
        }
    }
}

struct GaussianBlurInfo {
    sigma: Vector2F,
    bounds: RectI,
    render_target_id_y: RenderTargetId,
    render_target_id_x: RenderTargetId,
}
struct ColorMatrixInfo {
    bounds: RectI,
    render_target_id: RenderTargetId,
    filter: FeColorMatrix,
}