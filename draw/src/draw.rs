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
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
pub struct DrawContext<'a> {
    pub svg: &'a Svg,

    pub dpi: f32,

    #[cfg(feature="text")]
    pub font_cache: Option<FontCache<'a>>,
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
    pub fn new(svg: &'a Svg, fallback_fonts: &'a FontCollection) -> Self {
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
    pub fn compose(&'a self) -> Scene {
        self.compose_with_transform(Transform2F::default())
    }

    pub fn compose_with_transform(&'a self, transform: Transform2F) -> Scene {
        let mut options = DrawOptions::new(self);
        options.set_transform(transform);
        //options.view_box = Some(RectF::new(Vector2F::zero(), Vector2F::new(10., 10.)));
        self.compose_with_options(&options)
    }

    pub fn compose_with_options(&'a self, options: &DrawOptions) -> Scene {
        let mut scene = Scene::new();
        
        if let Some(vb) = self.view_box() {
            scene.set_view_box(options.transform * vb);
        }
        self.svg.root.draw_to(&mut scene, options);
        scene
    }

    pub fn compose_with_viewbox(&'a self, view_box: RectF) -> Scene {
        let options = DrawOptions::new(self);
        let mut scene = Scene::new();
        scene.set_view_box(options.transform * view_box);
        self.svg.root.draw_to(&mut scene, &options);
        scene
    }

    pub fn compose_to_with_transform(&'a self, scene: &mut Scene, transform: Transform2F) {
        let mut options = DrawOptions::new(self);
        options.transform = transform;
        self.svg.root.draw_to(scene, &options);
    }

    /// get the viewbox (computed if missing)
    pub fn view_box(&'a self) -> Option<RectF> {
        let options = BoundsOptions::new(self);
        
        if let Item::Svg(TagSvg { view_box: Some(r), width, height, .. }) = &*self.svg.root {
            if let Some(size) = Vector(
                width.unwrap_or(r.width),
                height.unwrap_or(r.height)
            ).try_resolve(&options) {
                return Some(RectF::new(Vector2F::zero(), size));
            }
        }
        self.svg.root.bounds(&options)
    }
}

#[derive(Clone, Debug)]
pub struct Options<'a> {
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

    pub clip_rule: FillRule,

    pub view_box: Option<RectF>,

    pub time: Time,

    pub font_size: f32,
    pub direction: TextFlow,

    pub lang: Option<Language>,
}
impl<'a> Options<'a> {
    pub fn new(ctx: &'a DrawContext<'a>) -> Options<'a> {
        Options {
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
            clip_rule: FillRule::EvenOdd,
            view_box: None,
            time: Time::start(),
            font_size: 20.,
            direction: TextFlow::LeftToRight,
            lang: None,
        }
    }
    pub fn has_stroke(&self) -> bool {
        self.opacity > 0.0 &&
        self.stroke_opacity > 0. &&
        !matches!(self.stroke, Paint::None)
    }
    pub fn has_fill(&self) -> bool {
        self.opacity > 0.0 &&
        self.fill_opacity > 0. &&
        !matches!(self.fill, Paint::None)
    }
    pub fn get_transform(&self) -> &Transform2F {
        &self.transform
    }
    pub fn set_transform(&mut self, transform: Transform2F) {
        self.transform = transform;
    }
    pub fn apply_transform(&mut self, transform: Transform2F) {
        self.transform = self.transform * transform;
    }
    pub fn apply(&self, attrs: &Attrs) -> Options<'a> {
        let mut stroke_style = self.stroke_style;
        if let Some(length) = attrs.stroke_width.resolve(self) {
            stroke_style.line_width = length;
        }
        Options {
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
                Axis::X => self.view_box.map(|r| r.width() * 0.01),
                Axis::Y => self.view_box.map(|r| r.height() * 0.01),
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
        
        self.apply_transform(Transform2F::from_scale(view_box.size().recip() * size) * Transform2F::from_translation(-view_box.origin()));
        self.view_box = Some(view_box);
    }
}

#[derive(Clone, Debug)]
pub struct DrawOptions<'a> {
    pub common: Options<'a>,
    pub clip_path: Option<(RectF, ClipPathId)>, //ClipPathAttr,
}
impl<'a> Deref for DrawOptions<'a> {
    type Target = Options<'a>;
    fn deref(&self) -> &Options<'a> {
        &self.common
    }
}
impl<'a> DerefMut for DrawOptions<'a> {
    fn deref_mut (&mut self) -> &mut Options<'a> {
        &mut self.common
    }
}

#[derive(Clone, Debug)]
pub struct BoundsOptions<'a> {
    pub common: Options<'a>,
    pub clip_rect: Option<RectF>,
}
impl<'a> Deref for BoundsOptions<'a> {
    type Target = Options<'a>;
    fn deref(&self) -> &Options<'a> {
        &self.common
    }
}
impl<'a> DerefMut for BoundsOptions<'a> {
    fn deref_mut (&mut self) -> &mut Options<'a> {
        &mut self.common
    }
}
impl<'a> BoundsOptions<'a> {
    pub fn new(ctx: &'a DrawContext<'a>) -> BoundsOptions<'a> {
        BoundsOptions {
            common: Options::new(ctx),
            clip_rect: None
        }
    }
    pub fn apply(&self, attrs: &Attrs) -> BoundsOptions<'a> {
        let common = self.common.apply(attrs);
        let clip_rect = match attrs.clip_path {
            Some(ClipPathAttr::Ref(ref id)) => {
                if let Some(Item::ClipPath(p)) = self.ctx.resolve(id).map(|t| &**t) {
                    let outline = p.resolve(&self);
                    let inner_rect = outline.bounds();
                    match self.clip_rect {
                        None => Some(inner_rect),
                        Some(outer_rect) => outer_rect.intersection(inner_rect),
                    }
                } else {
                    println!("clip path missing: {}", id);
                    None
                }
            }
            _ => self.clip_rect,
        };
        BoundsOptions { common, clip_rect }
    }
    pub fn bounds(&self, rect: RectF) -> Option<RectF> {
        let rect = if self.has_stroke() {
            Some(self.transform * rect.dilate(self.stroke_style.line_width))
        } else if self.has_fill() {
            Some(self.transform * rect)
        } else {
            None
        };
        if let Some(clip) = self.clip_rect {
            rect.and_then(|r| r.intersection(clip))
        } else {
            rect
        }
    }
}

impl<'a> DrawOptions<'a> {
    pub fn new(ctx: &'a DrawContext<'a>) -> DrawOptions<'a> {
        DrawOptions {
            common: Options::new(ctx),
            clip_path: None
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
    pub fn draw_transformed(&self, scene: &mut Scene, path: &Outline, transform: Transform2F) {
        let tr = self.transform * transform;
        let clip_path_id = self.clip_path.map(|(_, id)| id);
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
    pub fn apply(&self, scene: &mut Scene, attrs: &Attrs) -> DrawOptions<'a> {
        let common = self.common.apply(attrs);
        let clip_path = match attrs.clip_path {
            Some(ClipPathAttr::Ref(ref id)) => {
                if let Some(Item::ClipPath(p)) = self.ctx.resolve(id).map(|t| &**t) {
                    let outline = p.resolve(&common);
                    let clip_rect = outline.bounds();
                    println!("{:?}, {:?}, {:?}", p, outline, clip_rect);
                    // begin debug
                    /*
                    let paint = PaPaint::from_color(ColorU::new(255, 0, 255, 127));
                    let paint_id = scene.push_paint(&paint);
                    
                    let draw_path = DrawPath::new(outline.clone(), paint_id);
                    scene.push_draw_path(draw_path);
                    */
                    // end debug

                    let push_clip_path = |id: Option<ClipPathId>| {
                        let mut clip_path = ClipPath::new(outline);
                        clip_path.set_fill_rule(self.clip_rule);
                        clip_path.set_clip_path(id);
                        scene.push_clip_path(clip_path)
                    };

                    if let Some((rect, id)) = self.clip_path {
                        if let Some(intersection) = rect.intersection(clip_rect) {
                            Some((intersection, push_clip_path(Some(id))))
                        } else {
                            None
                        }
                    } else {
                        Some((clip_rect, push_clip_path(None)))
                    }
                } else {
                    println!("clip path missing: {}", id);
                    None
                }
            }
            _ => self.clip_path,
        };

        debug!("fill {:?} + {:?} -> {:?}", self.fill, attrs.fill, common.fill);
        debug!("stroke {:?} + {:?} -> {:?}", self.stroke, attrs.stroke, common.stroke);
        
        DrawOptions { common, clip_path }
    }
    pub fn bounds_options(&self) -> BoundsOptions<'a> {
        BoundsOptions {
            common: self.common.clone(),
            clip_rect: self.clip_path.map(|(rect, _)| rect)
        }
    }
}
