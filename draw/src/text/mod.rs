mod chunk;

use crate::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fmt;
use svg_text::{Font, FontCollection};
use chunk::{Chunk, ChunkLayout};
use crate::draw_glyph;
use unicode_segmentation::UnicodeSegmentation;

lazy_static! {
    static ref LATIN_MODERN: Font = Font::load(include_bytes!("../../../resources/latinmodern-math.otf"));
    static ref NOTO_NASKH_ARABIC: Font = Font::load(include_bytes!("/home/sebk/Rust/font/fonts/noto/NotoNaskhArabic-Regular.ttf"));
}


#[derive(Clone)]
pub struct FontCache {
    // TODO: use a lock-free map
    entries: Arc<Mutex<HashMap<String, Arc<FontCollection>>>>,
    fallback: Arc<FontCollection>,
}
impl fmt::Debug for FontCache {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FontCache")
    }
}
impl FontCache {
    pub fn new() -> Self {
        FontCache {
            entries: Arc::new(Mutex::new(HashMap::new())),
            fallback: Arc::new(FontCollection::from_fonts(vec![LATIN_MODERN.clone(), NOTO_NASKH_ARABIC.clone()])),
        }
    }
}

impl DrawItem for TagText {
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(&self.attrs);
        let state = TextState {
            pos: Vector2F::zero(),
            rot: 0.0
        };

        draw_items(scene, &options, &self.pos, &self.items, state, 0, None);
    }
    fn bounds(&self, options: &DrawOptions) -> Option<RectF> {
        None
    }
}

#[derive(Copy, Clone, Debug)]
struct TextState {
    pos: Vector2F,
    rot: f32,
}

fn chunk(scene: &mut Scene, options: &DrawOptions, s: &str, state: TextState, font: &FontCollection) -> Vector2F {
    println!("{} {:?}", s, state);
    let layout = Chunk::new(s, options.direction).layout(font);
    draw_layout(&layout, scene, &options, state)
}

fn draw_items(scene: &mut Scene, options: &DrawOptions, pos: &GlyphPos, items: &[Arc<Item>], mut state: TextState, mut char_idx: usize, parent_moves: Option<&Moves>) -> (TextState, usize) {
    let fallback = &*options.ctx.font_cache.fallback;
    let moves = Moves::new(pos, char_idx, parent_moves);

    for item in items.iter() {
        match **item {
            Item::String(ref s) => {
                for (idx, grapheme) in s.grapheme_indices(true) {
                    let num_chars = grapheme.chars().count();
                    if let Some(new_state) = moves.get(&options, num_chars, state, char_idx) {
                        state = new_state;
                        state.pos = state.pos + chunk(scene, options, grapheme, state, fallback);
                        char_idx += num_chars;
                    } else {
                        let part = &s[idx ..];
                        let num_chars = part.chars().count();
                        state.pos = state.pos + chunk(scene, options, part, state, fallback);
                        char_idx += num_chars;
                        break;
                    }
                }
            },
            Item::TSpan(ref span) => {
                let options = options.apply(&span.attrs);
                let (new_state, new_idx) = draw_items(scene, &options, &span.pos, &span.items, state, char_idx, Some(&moves));
                state = new_state;
                char_idx = new_idx;
            }
            _ => {}
        }
    }

    (state, char_idx)
}

fn draw_layout(layout: &ChunkLayout, scene: &mut Scene, options: &DrawOptions, state: TextState) -> Vector2F {
    for &(_, offset, ref sublayout) in &layout.parts {
        for &(ref glyph_variant, glyph_tr, glyph_offset) in &sublayout.glyphs {
            let chunk_tr = Transform2F::from_translation(state.pos) * Transform2F::from_rotation(deg2rad(state.rot))
                * Transform2F::from_scale(options.font_size)
                * Transform2F::from_translation(vec2f(offset + glyph_offset, 0.0));
            let tr = chunk_tr * glyph_tr;
            if let Some(ref svg) = glyph_variant.svg {
                draw_glyph(svg, scene, tr);
            } else {
                options.draw_transformed(scene, &glyph_variant.common.path, tr);
            }
        }
    }
    vec2f(layout.advance * options.font_size, 0.0)
}

fn slice<T>(o: &Option<OneOrMany<T>>) -> &[T] {
    o.as_ref().map(|l| l.as_slice()).unwrap_or(&[])
}

#[derive(Debug)]
struct Moves<'a> {
    x: &'a [LengthX],
    y: &'a [LengthY],
    dx: &'a [LengthX],
    dy: &'a [LengthY],
    rotate: &'a [f32],
    offset: usize,
    parent: Option<&'a Moves<'a>>,
}
impl<'a> Moves<'a> {
    fn new(pos: &'a GlyphPos, offset: usize, parent: Option<&'a Moves<'a>>) -> Self {
        Moves {
            x: slice(&pos.x),
            y: slice(&pos.y),
            dx: slice(&pos.dx),
            dy: slice(&pos.dy),
            rotate: slice(&pos.rotate),
            offset,
            parent,
        }
    }
    fn x(&self, idx: usize) -> Option<LengthX> {
        self.x.get(idx - self.offset).cloned().or_else(|| self.parent.and_then(|p| p.x(idx)))
    }
    fn y(&self, idx: usize) -> Option<LengthY> {
        self.y.get(idx - self.offset).cloned().or_else(|| self.parent.and_then(|p| p.y(idx)))
    }
    fn dx(&self, idx: usize) -> Option<LengthX> {
        self.dx.get(idx - self.offset).cloned().or_else(|| self.parent.and_then(|p| p.dx(idx)))
    }
    fn dy(&self, idx: usize) -> Option<LengthY> {
        self.dy.get(idx - self.offset).cloned().or_else(|| self.parent.and_then(|p| p.dy(idx)))
    }
    fn rotate(&self, idx: usize) -> Option<f32> {
        self.rotate.get(idx - self.offset).or(self.rotate.last()).cloned().or_else(|| self.parent.and_then(|p| p.rotate(idx)))
    }
    fn get<'o>(&self, options: &'o DrawOptions<'o>, num_chars: usize, state: TextState, idx: usize) -> Option<TextState> {
        let rel = |dx: Option<LengthX>, dy: Option<LengthY>| {
            let dx2: f32 = (idx + 1 .. idx + num_chars).flat_map(|idx| self.dx(idx).map(|l| l.resolve(options))).sum();
            let dy2: f32 = (idx + 1 .. idx + num_chars).flat_map(|idx| self.dy(idx).map(|l| l.resolve(options))).sum();
            vec2f(
                dx.map(|l| l.resolve(options)).unwrap_or(0.0) + dx2,
                dy.map(|l| l.resolve(options)).unwrap_or(0.0) + dy2
            )
        };
        let rot = |phi: Option<f32>| phi.unwrap_or(state.rot);

        match (self.x(idx), self.y(idx), self.dx(idx), self.dy(idx), self.rotate(idx)) {
            (None, None, None, None, None) => None,
            (None, None, dx, dy, phi) => Some(TextState { pos: state.pos + rel(dx, dy), rot: rot(phi)} ),
            (x, y, dx, dy, phi) => Some(TextState {
                pos: vec2f(
                    x.map(|l| l.resolve(options)).unwrap_or(state.pos.x()),
                    y.map(|l| l.resolve(options)).unwrap_or(state.pos.y()),
                ) + rel(dx, dy),
                rot: rot(phi)
            })
        }
    }
}
