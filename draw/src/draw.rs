use crate::prelude::*;
use pathfinder_content::{
    outline::{Outline},
    stroke::{OutlineStrokeToFill, StrokeStyle, LineCap, LineJoin},
    fill::{FillRule},
    dash::OutlineDash,
};
use pathfinder_renderer::{
    scene::{Scene, DrawPath, ClipPath, ClipPathId},
    paint::Paint as PaPaint,
};
use pathfinder_color::ColorU;
use svgtypes::{Length};
use std::sync::Arc;
use crate::gradient::BuildGradient;
#[cfg(feature="text")]
use crate::text::{FontCache};
use isolang::Language;
#[cfg(feature="text")]
use svg_text::FontCollection;
use std::rc::Rc;
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct DrawContext<'a> {
    pub svg: &'a Svg,

    pub dpi: f32,

    #[cfg(feature="text")]
    pub font_cache: Option<FontCache>,
}
impl<'a> DrawContext<'a> {
    pub fn new_without_fonts(svg: &'a Svg) -> Self {
        DrawContext {
            svg,
            dpi: 75.0,

            #[cfg(feature="text")]
            font_cache: None
        }
    }

    #[cfg(feature="text")]
    pub fn new(svg: &'a Svg, fallback_fonts: Arc<FontCollection>) -> Self {
        DrawContext {
            svg,
            dpi: 75.0,

            font_cache: Some(FontCache::new(fallback_fonts)),
        }
    }
    pub fn resolve(&self, id: &str) -> Option<&Arc<Item>> {
        self.svg.named_items.get(id)
    }
    pub fn resolve_href(&self, href: &str) -> Option<&Arc<Item>> {
        if href.starts_with("#") {
            self.resolve(&href[1..])
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct DrawOptions<'a> {
    pub ctx: &'a DrawContext<'a>,

    pub fill: Paint,
    pub fill_rule: FillRule,
    pub fill_opacity: f32,

    pub stroke: Paint,
    pub stroke_style: StrokeStyle,
    pub stroke_opacity: f32,
    pub stroke_dasharray: Option<Rc<[f32]>>,
    pub stroke_dashoffset: f32,

    pub opacity: f32,

    pub transform: Transform2F,

    pub clip_path: Option<ClipPath>, //ClipPathAttr,
    pub clip_rule: FillRule,

    pub view_box: Option<RectF>,

    pub time: Time,

    pub font_size: f32,
    pub direction: TextFlow,

    pub lang: Option<Language>,
}
impl<'a> DrawOptions<'a> {
    pub fn new(ctx: &'a DrawContext<'a>) -> DrawOptions<'a> {
        DrawOptions {
            ctx,
            opacity: 1.0,
            fill: Paint::black(),
            fill_rule: FillRule::EvenOdd,
            fill_opacity: 1.0,
            stroke: Paint::None,
            stroke_opacity: 1.0,
            stroke_style: StrokeStyle {
                line_width: 1.0,
                line_cap: LineCap::Butt,
                line_join: LineJoin::Bevel,
            },
            stroke_dasharray: None,
            stroke_dashoffset: 0.0,
            transform: Transform2F::from_scale(10.),
            clip_path: None,
            clip_rule: FillRule::EvenOdd,
            view_box: None,
            time: Time::start(),
            font_size: 20.,
            direction: TextFlow::LeftToRight,
            lang: None,
        }
    }
    pub fn bounds(&self, rect: RectF) -> Option<RectF> {
        let has_fill = matches!(*self,
            DrawOptions { ref fill, fill_opacity, .. }
            if fill.is_visible() && fill_opacity > 0.);
        let has_stroke = matches!(*self,
            DrawOptions { ref stroke, stroke_opacity, .. }
            if stroke.is_visible() && stroke_opacity > 0.
        );

        if has_stroke {
            Some(self.transform * rect.dilate(self.stroke_style.line_width))
        } else if has_fill {
            Some(self.transform * rect)
        } else {
            None
        }
    }
    fn resolve_paint(&self, paint: &Paint, opacity: f32) -> Option<PaPaint> {
        let opacity = opacity * self.opacity;
        match *paint {
            Paint::Color(ref c) => Some(PaPaint::from_color(c.color_u(opacity))),
            Paint::Ref(ref id) => match self.ctx.svg.named_items.get(id).map(|arc| &**arc) {
                Some(Item::LinearGradient(ref gradient)) => Some(PaPaint::from_gradient(gradient.build(self, opacity))),
                Some(Item::RadialGradient(ref gradient)) => Some(PaPaint::from_gradient(gradient.build(self, opacity))),
                r => {
                    dbg!(id, r);
                    None
                }
            }
            _ => None
        }
    }
    pub fn debug_outline(&self, scene: &mut Scene, path: &Outline, color: ColorU) {
        dbg!(path);
        let paint_id = scene.push_paint(&PaPaint::from_color(color));
        scene.push_draw_path(DrawPath::new(path.clone(), paint_id));
    }
    pub fn draw(&self, scene: &mut Scene, path: &Outline) {
        self.draw_transformed(scene, path, Transform2F::default());
    }
    fn clip_path_id(&self, scene: &mut Scene) -> Option<ClipPathId> {
        if let Some(ref clip_path) = self.clip_path {
            let mut clip_path = clip_path.clone();
            clip_path.set_fill_rule(self.clip_rule);
            
            // begin debug
            /*
            let paint = PaPaint::from_color(ColorU::new(255, 0, 255, 127));
            let paint_id = scene.push_paint(&paint);
            let draw_path = DrawPath::new(clip_path.outline().clone(), paint_id);
            scene.push_draw_path(draw_path);
            */
            // end debug

            Some(scene.push_clip_path(clip_path))
        } else {
            None
        }
    }
    pub fn draw_transformed(&self, scene: &mut Scene, path: &Outline, transform: Transform2F) {
        let tr = self.transform * transform;
        let clip_path_id = self.clip_path_id(scene);
        if let Some(ref fill) = self.resolve_paint(&self.fill, self.fill_opacity) {
            let outline = path.clone().transformed(&tr);
            let paint_id = scene.push_paint(fill);
            let mut draw_path = DrawPath::new(outline, paint_id);
            draw_path.set_fill_rule(self.fill_rule);
            draw_path.set_clip_path(clip_path_id);
            scene.push_draw_path(draw_path);
        }
        if let Some(ref stroke) = self.resolve_paint(&self.stroke, self.stroke_opacity) {
            if self.stroke_style.line_width > 0. {
                let paint_id = scene.push_paint(stroke);

                let mut outline = Cow::Borrowed(path);
                if let Some(ref dash) = self.stroke_dasharray {
                    let mut dash = OutlineDash::new(&path, dash, self.stroke_dashoffset);
                    dash.dash();
                    outline = Cow::Owned(dash.into_outline());
                }
                let mut stroke = OutlineStrokeToFill::new(&outline, self.stroke_style);
                stroke.offset();
                let path = stroke.into_outline();
                let mut draw_path = DrawPath::new(path.transformed(&tr), paint_id);
                draw_path.set_clip_path(clip_path_id);
                scene.push_draw_path(draw_path);
            }
        }
    }
    pub fn transform(&mut self, transform: Transform2F) {
        self.transform = self.transform * transform;
    }
    pub fn apply(&self, attrs: &Attrs) -> DrawOptions<'a> {
        let mut stroke_style = self.stroke_style;
        if let Some(length) = attrs.stroke_width.resolve(self) {
            stroke_style.line_width = length;
        }
        let mut new = DrawOptions {
            clip_path: None,
            clip_rule: attrs.clip_rule.unwrap_or(self.clip_rule),
            opacity: attrs.opacity.resolve(self).unwrap_or(1.0),
            transform: self.transform * attrs.transform.resolve(self),
            fill: attrs.fill.resolve(self),
            fill_rule: attrs.fill_rule.unwrap_or(self.fill_rule),
            fill_opacity: attrs.fill_opacity.resolve(self).unwrap_or(self.fill_opacity),
            stroke: attrs.stroke.resolve(self),
            stroke_style,
            stroke_opacity: attrs.stroke_opacity.resolve(self).unwrap_or(self.stroke_opacity),
            stroke_dasharray: attrs.stroke_dasharray.resolve(self),
            direction: attrs.direction.unwrap_or(self.direction),
            font_size: attrs.font_size.resolve(self).unwrap_or(self.font_size),
            lang: attrs.lang.or(self.lang),
            .. *self
        };
        new.clip_path = match attrs.clip_path {
            None => self.clip_path.clone(),
            Some(ClipPathAttr::None) => None,
            Some(ClipPathAttr::Ref(ref id)) => {
                if let Some(Item::ClipPath(p)) = self.ctx.resolve(id).map(|t| &**t) {
                    let outline = p.resolve(&new);
                    Some(ClipPath::new(outline))
                } else {
                    println!("clip path missing: {}", id);
                    None
                }
            }
        };

        debug!("fill {:?} + {:?} -> {:?}", self.fill, attrs.fill, new.fill);
        debug!("stroke {:?} + {:?} -> {:?}", self.stroke, attrs.stroke, new.stroke);
        new
    }
    pub fn resolve_length(&self, length: Length) -> Option<f32> {
        let scale = match length.unit {
            LengthUnit::None => 1.0,
            LengthUnit::Cm => self.ctx.dpi * (1.0 / 2.54),
            LengthUnit::Em => unimplemented!(),
            LengthUnit::Ex => unimplemented!(),
            LengthUnit::In => self.ctx.dpi,
            LengthUnit::Mm => self.ctx.dpi * (1.0 / 25.4),
            LengthUnit::Pc => unimplemented!(),
            LengthUnit::Percent => return None,
            LengthUnit::Pt => self.ctx.dpi * (1.0 / 75.),
            LengthUnit::Px => 1.0
        };
        Some(length.num as f32 * scale)
    }
    pub fn resolve_length_along(&self, length: Length, axis: Axis) -> Option<f32> {
        let scale = match length.unit {
            LengthUnit::None => 1.0,
            LengthUnit::Cm => self.ctx.dpi * (1.0 / 2.54),
            LengthUnit::Em => unimplemented!(),
            LengthUnit::Ex => unimplemented!(),
            LengthUnit::In => self.ctx.dpi,
            LengthUnit::Mm => self.ctx.dpi * (1.0 / 25.4),
            LengthUnit::Pc => unimplemented!(),
            LengthUnit::Percent => return match axis {
                Axis::X => self.view_box.map(|r| r.width()),
                Axis::Y => self.view_box.map(|r| r.height()),
            },
            LengthUnit::Pt => self.ctx.dpi * (1.0 / 75.),
            LengthUnit::Px => 1.0
        };
        Some(length.num as f32 * scale)
    }
    pub fn apply_viewbox(&mut self, width: Option<LengthX>, height: Option<LengthY>, view_box: &Rect) {
        let view_box = view_box.resolve(self);
        let width = width.and_then(|l| l.try_resolve(self)).unwrap_or(view_box.width());
        let height = height.and_then(|l| l.try_resolve(self)).unwrap_or(view_box.height());
        let size = vec2f(width, height);
        
        self.transform(Transform2F::from_scale(view_box.size().recip() * size) * Transform2F::from_translation(-view_box.origin()));
        self.view_box = Some(view_box);
    }
}
