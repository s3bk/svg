use font::{Glyph, GlyphId, SvgGlyph};
use pathfinder_geometry::{
    vector::{Vector2F, vec2f},
    transform2d::Transform2F,
    rect::RectF,
};
use std::sync::Arc;
use itertools::Itertools;


#[derive(Clone)]
pub struct Font(Arc<dyn font::Font + Sync + Send>);
impl Font {
    pub fn load(data: &[u8]) -> Font {
        Font(Arc::from(font::parse(data)))
    }
}
impl std::ops::Deref for Font {
    type Target = dyn font::Font;
    fn deref(&self) -> &dyn font::Font {
        &*self.0
    }
}

#[derive(Clone)]
pub struct FontCollection {
    fonts: Vec<Font>
}
impl FontCollection {
    pub fn new() -> FontCollection {
        FontCollection { fonts: vec![] }
    }
    pub fn from_font(font: Font) -> FontCollection {
        FontCollection { fonts: vec![font] }
    }
    pub fn from_fonts(fonts: Vec<Font>) -> FontCollection {
        FontCollection { fonts }
    }
    pub fn add_font(&mut self, font: Font) {
        self.fonts.push(font);
    }
}

impl FontCollection {
    pub fn layout_run(&self, string: &str, rtl: bool) -> Layout {
        use std::iter::once;
        let fonts = &*self.fonts;

        let mut gids: Vec<(u32, GlyphId)> = string.chars().filter_map(|c| {
            fonts.iter().enumerate().filter_map(move |(font_idx, font)| font.gid_for_unicode_codepoint(c as u32).map(|gid| (font_idx as u32, gid))).next()
        }).collect();
        
        let mut pos = 0;
    'a: while let Some(&(font_idx, first)) = gids.get(pos) {
            pos += 1;
            let font = &fonts[font_idx as usize];
            if let Some(gsub) = font.get_gsub() {
                if let Some(subs) = gsub.substitutions(first) {
                    for (sub, glyph) in subs {
                        if let Some(len) = sub.matches(
                            gids[pos ..].iter()
                            .take_while(|&&(gid_font_idx, gid)| gid_font_idx == font_idx)
                            .map(|&(_, gid)| gid))
                        {
                            gids.splice(pos-1 .. pos+len, once((font_idx, glyph)));
                            continue 'a;
                        }
                    }
                }
            }
        }
        
        let mut last_gid = None;
        let mut offset = 0.0;
        let mut glyphs = Vec::with_capacity(gids.len());
        for &(font_idx, gid) in gids.iter() {
            let face = &fonts[font_idx as usize];
            if let Some(glyph) = face.glyph(gid) {
                if let Some(left) = last_gid.replace(gid) {
                    offset += face.kerning(left, gid);
                }
                let tr = Transform2F::from_translation(vec2f(0.0, 0.0)) * Transform2F::from_scale(vec2f(1.0, -1.0)) * face.font_matrix();
                let advance = tr.m11() * glyph.metrics.advance;
                let (advance, glyph_offset) = match rtl {
                    false => (advance, offset),
                    true => (- advance, offset - advance)
                };

                let svg = face.svg_glyph(gid).cloned();
                let glyph = GlyphVariant {
                    common: glyph,
                    svg
                };
                glyphs.push((glyph, tr, glyph_offset));

                offset = offset + advance;
            }
        }


        let bbox: RectF = glyphs.iter().map(|&(ref glyph, tr, offset)| Transform2F::from_translation(Vector2F::new(offset, 0.0)) * tr * glyph.common.path.bounds()).fold1(|a, b| a.union_rect(b)).unwrap_or_default();
        let (font_bounding_box_ascent, font_bounding_box_descent) = fonts.iter().filter_map(
            |f| f.vmetrics().map(|m| (m.ascent, m.descent))
        ).fold1(|(a1, d1), (a2, d2)| (a1.max(a2), d1.min(d2))).unwrap_or((0., 0.));

        let metrics = TextMetrics {
            advance: offset,
            font_bounding_box_ascent,
            font_bounding_box_descent,
        };
        Layout {
            bbox,
            glyphs,
            metrics,
        }
    }
}
pub struct GlyphVariant {
    pub common: Glyph,
    pub svg: Option<SvgGlyph>
}

pub struct Layout {
    pub metrics: TextMetrics,
    pub bbox: RectF,
    pub glyphs: Vec<(GlyphVariant, Transform2F, f32)>
}

pub struct TextMetrics {
    pub advance: f32,
    pub font_bounding_box_ascent: f32,
    pub font_bounding_box_descent: f32
}
