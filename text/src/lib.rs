#[macro_use] extern crate log;

use font::{Glyph, GlyphId, SvgGlyph, gsub::{Gsub, Substitution, Tag, LanguageSystem}};
use pathfinder_geometry::{
    vector::{Vector2F, vec2f},
    transform2d::Transform2F,
    rect::RectF,
};
use std::sync::Arc;
use itertools::Itertools;
use unicode_segmentation::UnicodeSegmentation;
use unicode_joining_type::{get_joining_type, JoiningType};

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

// returns (next pos, length change of glyphs)
fn apply_subs<'a, 'b>(glyphs: &'a mut Vec<GlyphId>, pos: usize, subs: impl Iterator<Item=&'b Substitution>) -> (usize, isize) {
    for sub in subs {
        let first = glyphs[pos].0 as u16;
        match *sub {
            Substitution::Single(ref map) => {
                if let Some(&replacement) = map.get(&first) {
                    debug!("replace gid {:?} with {:?}", glyphs[pos], GlyphId(replacement as u32));
                    glyphs[pos] = GlyphId(replacement as u32);
                    return (pos + 1, 0);
                }
            }
            Substitution::Ligatures(ref map) => {
                if let Some(subs) = map.get(&first) {
                    for &(ref sub, glyph) in subs {
                        if let Some(len) = sub.matches(glyphs[pos + 1 ..].iter().cloned()) {
                            debug!("ligature {}..{} with {:?}", pos, pos+len+1, GlyphId(glyph as u32));
                            glyphs.splice(pos .. pos+len+1, std::iter::once(GlyphId(glyph as u32)));
                            return (pos + 1, -(len as isize));
                        }
                    }
                }
            }
        }
    }
    (pos + 1, 0)
}

#[derive(Debug, Copy, Clone)]
enum GlyphLocation {
    Initial,
    Middle,
    Final,
    Isolated,
}
impl GlyphLocation {
    fn join(self, next: GlyphLocation) -> GlyphLocation {
        match (self, next) {
            (GlyphLocation::Initial, GlyphLocation::Final) => GlyphLocation::Isolated,
            (GlyphLocation::Initial, GlyphLocation::Middle) => GlyphLocation::Initial,
            (GlyphLocation::Middle, GlyphLocation::Final) => GlyphLocation::Final,
            (GlyphLocation::Isolated, GlyphLocation::Isolated) => GlyphLocation::Isolated,
            _ => GlyphLocation::Isolated
        }
    }
}

#[derive(Debug)]
struct MetaGlyph {
    codepoint: char,
    joining_type: JoiningType,
    location: GlyphLocation
}
impl MetaGlyph {
    fn new(codepoint: char) -> MetaGlyph {
        MetaGlyph {
            codepoint,
            joining_type: get_joining_type(codepoint),
            location: GlyphLocation::Isolated
        }
    }
}

fn compute_joining(meta: &mut [MetaGlyph]) {
    for (prev, next) in meta.iter_mut().tuples() {
        match prev.joining_type {
            JoiningType::LeftJoining | JoiningType::DualJoining | JoiningType::JoinCausing => {
                match next.joining_type {
                    JoiningType::RightJoining | JoiningType::DualJoining | JoiningType::JoinCausing => {
                        next.location = GlyphLocation::Final;
                    
                        prev.location = match prev.location {
                            GlyphLocation::Isolated => GlyphLocation::Initial,
                            GlyphLocation::Final => GlyphLocation::Middle,
                            loc => loc,
                        }
                    }
                    JoiningType::LeftJoining | JoiningType::NonJoining => {
                        prev.location = match prev.location {
                            GlyphLocation::Initial => GlyphLocation::Isolated,
                            loc => loc,
                        };
                    }
                    JoiningType::Transparent => {}
                }
            }
            JoiningType::RightJoining | JoiningType::NonJoining => {
                prev.location = match prev.location {
                    GlyphLocation::Initial => GlyphLocation::Isolated,
                    loc => loc,
                };
            }
            JoiningType::Transparent => {}
        }
    }
}

fn sub_pass<F, G>(gsub: &Gsub, lang: &LanguageSystem, meta: &[MetaGlyph], gids: &mut Vec<GlyphId>, filter_fn: F)
    where F: Fn(&MetaGlyph) -> G, G: Fn(Tag) -> bool
{
    let mut pos = 0;
    let mut meta_pos = 0isize;
    while let Some(m) = meta.get(meta_pos as usize) {
        debug!("pos {}, meta_pos: {}, gids[pos] = {:?}, meta[meta_pos] = {:?}", pos, meta_pos, gids[pos], m);
        let (next_pos, delta) = apply_subs(gids, pos, gsub.subs(lang, filter_fn(m)));
        meta_pos += (next_pos - pos) as isize - delta;
        pos = next_pos;
    }
}

fn process_chunk(font: &Font, language: Option<&str>, rtl: bool, meta: &[MetaGlyph], state: &mut State) {
    for g in meta {
        debug!("[\u{2068}{}\u{2069} 0x{:x}]", g.codepoint, g.codepoint as u32);
    }
    let mut gids: Vec<GlyphId> = meta.iter()
        .map(|m| font.gid_for_unicode_codepoint(m.codepoint as u32).unwrap())
        .collect();

    if let Some(gsub) = font.get_gsub() {
        if let Some(lang) = language.and_then(|s| gsub.language(s)).or(gsub.default_language()) {
            sub_pass(gsub, lang, meta, &mut gids, |m| {
                let arabic_tag = match m.location {
                    GlyphLocation::Isolated => Tag(*b"isol"),
                    GlyphLocation::Initial => Tag(*b"init"),
                    GlyphLocation::Final => Tag(*b"fina"),
                    GlyphLocation::Middle => Tag(*b"medi")
                };
                move |tag: Tag| tag == arabic_tag
            });
            sub_pass(gsub, lang, meta, &mut gids, |m| |tag| [Tag(*b"rlig"), Tag(*b"liga")].contains(&tag));
        }
    }
    
    let mut last_gid = None;
    for gid in gids {
        if let Some(glyph) = font.glyph(gid) {
            let kerning = last_gid.replace(gid).map(|left| font.kerning(left, gid)).unwrap_or_default();
            let tr = Transform2F::from_translation(vec2f(0.0, 0.0)) * Transform2F::from_scale(vec2f(1.0, -1.0)) * font.font_matrix();
            let advance = tr.m11() * glyph.metrics.advance + kerning;
            let (advance, glyph_offset) = match rtl {
                false => (advance, state.offset + kerning),
                true => (- advance, state.offset - advance)
            };

            let svg = font.svg_glyph(gid).cloned();
            let glyph = GlyphVariant {
                common: glyph,
                svg
            };

            state.offset += advance;
            state.glyphs.push((glyph, tr, glyph_offset));
        }
    }
}

struct State {
    glyphs: Vec<(GlyphVariant, Transform2F, f32)>,
    offset: f32
}

impl FontCollection {
    pub fn layout_run(&self, string: &str, rtl: bool, lang: Option<&str>) -> Layout {
        let fonts = &*self.fonts;

        let mut state = State {
            offset: 0.0,
            glyphs: Vec::with_capacity(string.len())
        };

        let font_for_text = |text: &str| fonts.iter()
            .filter(|font|
                text.chars().all(|c| font.gid_for_unicode_codepoint(c as u32).is_some())
            ).next();

        // we process each word separately to improve the visual appearance by trying to render a word in a single font
        for word in string.split_word_bounds() {
            // do stuff… borrowed from allsorts
            let mut meta: Vec<MetaGlyph> = word.chars().map(|c| MetaGlyph::new(c)).collect();
            compute_joining(&mut meta);
            
            // try to find a font that has all glyphs
            if let Some(font) = font_for_text(word) {
                process_chunk(font, lang, rtl, &meta, &mut state);
            } else {
                let mut start = 0;
                let mut current_font = None;
                for (idx, grapheme) in word.grapheme_indices(true) {
                    if let Some(font) = font_for_text(grapheme) {
                        if Some(font as *const _) != current_font.map(|f| f as *const _) && idx > 0 {
                            // flush so far
                            process_chunk(font, lang, rtl, &meta[start .. idx], &mut state);
                            start = idx;
                        }
                        current_font = Some(font);
                    }
                }
                if let Some(font) = current_font {
                    process_chunk(font, lang, rtl, &meta[start ..], &mut state);
                }
            }
        }


        let bbox: RectF = state.glyphs.iter().map(|&(ref glyph, tr, offset)| Transform2F::from_translation(Vector2F::new(offset, 0.0)) * tr * glyph.common.path.bounds()).fold1(|a, b| a.union_rect(b)).unwrap_or_default();
        let (font_bounding_box_ascent, font_bounding_box_descent) = fonts.iter().filter_map(
            |f| f.vmetrics().map(|m| (m.ascent, m.descent))
        ).fold1(|(a1, d1), (a2, d2)| (a1.max(a2), d1.min(d2))).unwrap_or((0., 0.));

        let metrics = TextMetrics {
            advance: state.offset,
            font_bounding_box_ascent,
            font_bounding_box_descent,
        };
        Layout {
            bbox,
            glyphs: state.glyphs,
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
