#[macro_use] extern crate log;

use font::{Glyph, GlyphId,
    opentype::{
        OpenTypeFont, Tag,
        gsub::{GSub, Substitution, LanguageSystem},
        gdef::MarkClass,
    },
};
#[cfg(feature="svg")]
use font::SvgGlyph;

pub use font::FontError;
use pathfinder_geometry::{
    vector::{Vector2F, vec2f},
    transform2d::Transform2F,
    rect::RectF,
};
use std::sync::Arc;
use std::fmt::{self, Debug};
use std::ops::Deref;
use itertools::Itertools;
use unic_segment::{WordBoundIndices, GraphemeIndices};
use unic_ucd_category::GeneralCategory;
use unicode_joining_type::{get_joining_type, JoiningType};
use isolang::Language;

#[derive(Clone)]
pub struct Font(Arc<dyn font::Font + Sync + Send>);
impl Font {
    pub fn load(data: &[u8]) -> Font {
        Font(Arc::from(font::parse(data).unwrap()))
    }
}
impl std::ops::Deref for Font {
    type Target = dyn font::Font + Sync + Send;
    fn deref(&self) -> &(dyn font::Font + Sync + Send) {
        &*self.0
    }
}
impl Debug for Font {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name().full_name.as_ref().map(|s| s.as_str()).unwrap_or_default())
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
impl Deref for FontCollection {
    type Target = [Font];
    #[inline]
    fn deref(&self) -> &[Font] {
        &self.fonts
    }
}

// returns (next pos, length change of glyphs)
fn apply_subs<'a, 'b>(glyphs: &'a mut Vec<(usize, GlyphId)>, pos: usize, subs: impl Iterator<Item=&'b Substitution>) -> (usize, isize) {
    for sub in subs {
        let (first_idx, GlyphId(first)) = glyphs[pos];
        match *sub {
            Substitution::Single(ref map) => {
                if let Some(&replacement) = map.get(&(first as u16)) {
                    debug!("replace gid {:?} with {:?}", glyphs[pos], GlyphId(replacement as u32));
                    glyphs[pos] = (first_idx, GlyphId(replacement as u32));
                    return (pos + 1, 0);
                }
            }
            Substitution::Ligatures(ref map) => {
                if let Some(subs) = map.get(&(first as u16)) {
                    for &(ref sub, glyph) in subs {
                        if let Some(len) = sub.matches(glyphs[pos + 1 ..].iter().map(|&(_, gid)| gid)) {
                            debug!("ligature {}..{} with {:?}", pos, pos+len+1, GlyphId(glyph as u32));
                            glyphs.splice(pos .. pos+len+1, std::iter::once((first_idx, GlyphId(glyph as u32))));
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
    location: GlyphLocation,
    category: GeneralCategory,
    idx: usize,
}
impl MetaGlyph {
    fn new(codepoint: char, idx: usize) -> MetaGlyph {
        MetaGlyph {
            codepoint,
            joining_type: get_joining_type(codepoint),
            location: GlyphLocation::Isolated,
            category: GeneralCategory::of(codepoint),
            idx
        }
    }
}

fn compute_joining(meta: &mut [MetaGlyph]) {
    for i in 1 .. meta.len() {
        let (prev, next) = meta.split_at_mut(i);
        let prev = &mut prev[i - 1];
        let next = &mut next[0];
        
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

fn sub_pass<F, G>(gsub: &GSub, lang: &LanguageSystem, meta: &[MetaGlyph], gids: &mut Vec<(usize, GlyphId)>, filter_fn: F)
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

fn process_chunk(font: &Font, font_idx: usize, language: Option<Tag>, rtl: bool, meta: &[MetaGlyph], state: &mut State) {
    if let Some(fm) = font.vmetrics() {
        let s = font.font_matrix().m22();
        let vm = VMetrics {
            ascent: fm.ascent * s,
            descent: fm.descent * s
        };
        state.vmetrics = match state.vmetrics {
            None => Some(vm),
            Some(m1) => Some(VMetrics { ascent: m1.ascent.max(vm.ascent), descent: m1.descent.min(vm.descent) })
        };
    }

    for g in meta {
        debug!("at byte {} [\u{2068}{}\u{2069} 0x{:x}]", g.idx, g.codepoint, g.codepoint as u32);
    }
    // (codepoint idx, glyph id)
    let mut gids: Vec<(usize, GlyphId)> = meta.iter()
        .filter(|&m| match m.category {
            GeneralCategory::Format => false,
            _ => true
        })
        .map(|m| (m.idx, font.gid_for_unicode_codepoint(m.codepoint as u32).unwrap()))
        .collect();

    let otf = font.downcast_ref::<OpenTypeFont>();
    let gsub = otf.and_then(|f| f.gsub.as_ref());
    let gdef = otf.and_then(|f| f.gdef.as_ref());
    let gpos = otf.and_then(|f| f.gpos.as_ref());

    if let Some(gsub) = gsub {
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
    for (index, gid) in gids {
        if let Some(glyph) = font.glyph(gid) {
            let mark = match (gdef.and_then(|gdef| gdef.mark_class(gid.0 as u16)).unwrap_or(MarkClass::Unassigned), last_gid) {
                (MarkClass::Mark, Some(last)) => {
                    gpos.and_then(|gpos| gpos.get_mark_to_base(last, gid))
                },
                _ => None
            };

            let (advance, offset) = match mark {
                None => {
                    let kerning = font.font_matrix() * vec2f(last_gid.replace(gid).map(|left| font.kerning(left, gid)).unwrap_or_default(), 0.0);
                    let advance = font.font_matrix() * vec2f(glyph.metrics.advance, 0.0) + kerning;
                    match rtl {
                        false => (advance, state.offset + kerning),
                        true => (advance * vec2f(-1.0, 1.0), state.offset - advance)
                    }
                }
                Some((dx, dy)) => {
                    let delta = font.font_matrix() * vec2f(dx as f32, dy as f32);
                    (Vector2F::zero(), state.offset + delta)
                }
            };

            let transform = Transform2F::from_scale(vec2f(1.0, -1.0)) * font.font_matrix();
            state.offset += advance;
            state.glyphs.push(LayoutGlyph { gid, transform, offset, index, font_idx });
        }
    }
}

pub struct LayoutGlyph {
    pub gid: GlyphId,
    pub transform: Transform2F,
    pub offset: Vector2F,

    // byte index of this glyph in the input
    pub index: usize,
    
    // index of the font in the fontcollection this glyph belongs to
    pub font_idx: usize,
}

#[derive(Copy, Clone)]
struct VMetrics {
    ascent: f32,
    descent: f32,
}

struct State {
    // (variant, glyph transform, base offset, str offset)
    glyphs: Vec<LayoutGlyph>,
    offset: Vector2F,
    vmetrics: Option<VMetrics>,
}

fn font_for_text<'a>(fonts: &'a [Font], text: &str, meta: &[MetaGlyph]) -> Option<(usize, &'a Font)> {
    fonts.iter().enumerate()
        .filter(|(_, font)|
            text.chars().zip(meta).all(|(c, m)| {
                match m.category {
                    GeneralCategory::Format => true,
                    _ => font.gid_for_unicode_codepoint(c as u32).is_some()
                }
            })
        ).next()
}

impl FontCollection {
    pub fn layout_run(&self, string: &str, rtl: bool, lang: Option<Language>) -> Layout {
        let lang = lang.and_then(tags::lang_to_tag).or_else(|| guess_lang(string));

        let fonts = &*self.fonts;
        if fonts.len() == 0 {
            warn!("no fonts!");
        }

        let mut state = State {
            offset: Vector2F::zero(),
            glyphs: Vec::with_capacity(string.len()),
            vmetrics: None,
        };

        // we process each word separately to improve the visual appearance by trying to render a word in a single font
        for (word_off, word) in WordBoundIndices::new(string) {
            // do stuff… borrowed from allsorts
            let mut meta: Vec<MetaGlyph> = word.char_indices().map(|(idx, c)| MetaGlyph::new(c, word_off+idx)).collect();
            compute_joining(&mut meta);
            
            // try to find a font that has all glyphs
            if let Some((font_idx, font)) = font_for_text(fonts, word, &meta) {
                process_chunk(font, font_idx, lang, rtl, &meta, &mut state);
            } else {
                let mut start = 0;
                let mut meta_idx = 0;
                let mut current_font = None;
                info!("word: {}", word);
                for (idx, grapheme) in GraphemeIndices::new(word) {
                    let meta_len = grapheme.chars().count();
                    if let Some((font_idx, font)) = font_for_text(fonts, grapheme, &meta[meta_idx .. meta_idx + meta_len]) {
                        if Some(font_idx) != current_font.map(|(i, _)| i) && idx > 0 {
                            // flush so fart.0
                            process_chunk(font, font_idx, lang, rtl, &meta[start .. idx], &mut state);
                            start = idx;
                        }
                        current_font = Some((font_idx, font));
                    } else {
                        current_font = None;
                        warn!("no font for {:?}", grapheme);
                    }
                    meta_idx += meta_len;
                }
                if let Some((font_idx, font)) = current_font {
                    process_chunk(font, font_idx, lang, rtl, &meta[meta_idx ..], &mut state);
                }
            }
        }

        let (font_bounding_box_ascent, font_bounding_box_descent) = fonts.iter().filter_map(
            |f| {
                let s = f.font_matrix().m22();
                f.vmetrics().map(|m| (s * m.ascent, s * m.descent))
            }
        ).fold1(|(a1, d1), (a2, d2)| (a1.max(a2), d1.min(d2))).unwrap_or((0., 0.));

        let vmetrics = state.vmetrics.unwrap_or(VMetrics { ascent: 0.0, descent: 0.0 });
        let metrics = TextMetrics {
            advance: state.offset,
            font_bounding_box_ascent,
            font_bounding_box_descent,
            ascent: vmetrics.ascent,
            descent: vmetrics.descent,
        };

        Layout {
            glyphs: state.glyphs,
            metrics,
        }
    }

    pub fn layout_bbox(&self, layout: &Layout) -> RectF {
        layout.glyphs.iter()
            .map(|g| {
                let glyph = self[g.font_idx].glyph(g.gid).unwrap();
                let bounds = glyph.path.bounds();
                Transform2F::from_translation(g.offset) * g.transform * bounds
            })
            .fold1(|a, b| a.union_rect(b)).unwrap_or_default()
    }
}

mod tags;

fn guess_lang(text: &str) -> Option<Tag> {
    whatlang::detect(text)
        .and_then(|info| Language::from_639_3(info.lang().code()))
        .and_then(tags::lang_to_tag)
}

pub struct GlyphVariant {
    pub common: Glyph,

    #[cfg(feature="svg")]
    pub svg: Option<SvgGlyph>
}

pub struct Layout {
    pub metrics: TextMetrics,
    pub glyphs: Vec<LayoutGlyph>
}

pub struct TextMetrics {
    pub advance: Vector2F,
    pub font_bounding_box_ascent: f32,
    pub font_bounding_box_descent: f32,
    pub ascent: f32,
    pub descent: f32,
}

enum HAlign {
    Left,
    Center,
    Right
}

enum VAlign {
    Baseline
}