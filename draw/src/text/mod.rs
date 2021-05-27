mod chunk;

use crate::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fmt;
use svg_text::{Font, FontCollection};
use chunk::{Chunk, ChunkLayout};
use crate::draw_glyph;
use unic_segment::{WordBounds, GraphemeIndices};

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
    pub fn new(fallback: Arc<FontCollection>) -> Self {
        FontCache {
            entries: Arc::new(Mutex::new(HashMap::new())),
            fallback,
        }
    }
}

impl DrawItem for TagText {
    fn draw_to(&self, scene: &mut Scene, options: &DrawOptions) {
        let options = options.apply(scene, &self.attrs);
        let state = TextState {
            pos: Vector2F::zero(),
            rot: 0.0
        };

        if let Some(ref font_cache) = options.ctx.font_cache {
            draw_items(scene, &options, font_cache, &self.pos, &self.items, state, 0, None);
        }
    }
    fn bounds(&self, options: &BoundsOptions) -> Option<RectF> {
        None
    }
}

#[derive(Copy, Clone, Debug)]
struct TextState {
    pos: Vector2F,
    rot: f32,
}
impl TextState {
    fn apply_move(self, m: Move) -> TextState {
        let x = m.abs_x.unwrap_or(self.pos.x());
        let y = m.abs_y.unwrap_or(self.pos.y());
        let rot = m.rot.unwrap_or(self.rot);
        TextState {
            pos: vec2f(x, y) + m.rel,
            rot
        }
    }
}

fn chunk(scene: &mut Scene, options: &DrawOptions, s: &str, state: TextState, font: &FontCollection) -> Vector2F {
    debug!("{} {:?}", s, state);
    let layout = Chunk::new(s, options.direction).layout(font, options.lang);
    draw_layout(&layout, scene, &options, state)
}

fn draw_items(scene: &mut Scene, options: &DrawOptions, font_cache: &FontCache, pos: &GlyphPos, items: &[Arc<Item>], mut state: TextState, mut char_idx: usize, parent_moves: Option<&Moves>) -> (TextState, usize) {
    let fallback = &font_cache.fallback;
    let moves = Moves::new(pos, char_idx, parent_moves);

    for item in items.iter() {
        match **item {
            Item::String(ref s) if s.len() > 0 => {
                let mut start = 0;
                for (idx, grapheme) in GraphemeIndices::new(s) {
                    let num_chars = grapheme.chars().count();
                    if let Some(next_move) = moves.get(&options, num_chars, char_idx) {
                        if idx > 0 {
                            state.pos = state.pos + chunk(scene, options, &s[start .. idx], state, fallback);
                        }
                        start = idx;
                        state = state.apply_move(next_move);
                        char_idx += num_chars;
                    }
                }

                let part = &s[start ..];
                let num_chars = part.chars().count();
                state.pos = state.pos + chunk(scene, options, part, state, fallback);
                char_idx += num_chars;
            },
            Item::TSpan(ref span) => {
                let options = options.apply(scene, &span.attrs);
                let (new_state, new_idx) = draw_items(scene, &options, font_cache, &span.pos, &span.items, state, char_idx, Some(&moves));
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
        for glyph in &sublayout.glyphs {
            let chunk_tr = Transform2F::from_translation(state.pos) * Transform2F::from_rotation(deg2rad(state.rot))
                * Transform2F::from_scale(options.font_size)
                * Transform2F::from_translation(offset + glyph.offset);
            let tr = chunk_tr * glyph.transform;
            if let Some(ref svg) = glyph.glyph.svg {
                draw_glyph(svg, scene, tr);
            } else {
                options.draw_transformed(scene, &glyph.glyph.common.path, tr);
            }
        }
    }
    layout.advance * options.font_size
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
    fn get<'o>(&self, options: &'o DrawOptions<'o>, num_chars: usize, idx: usize) -> Option<Move> {
        let rel = |dx: Option<LengthX>, dy: Option<LengthY>| {
            let dx2: f32 = (idx + 1 .. idx + num_chars).flat_map(|idx| self.dx(idx).map(|l| l.resolve(options))).sum();
            let dy2: f32 = (idx + 1 .. idx + num_chars).flat_map(|idx| self.dy(idx).map(|l| l.resolve(options))).sum();
            vec2f(
                dx.map(|l| l.resolve(options)).unwrap_or(0.0) + dx2,
                dy.map(|l| l.resolve(options)).unwrap_or(0.0) + dy2
            )
        };

        match (self.x(idx), self.y(idx), self.dx(idx), self.dy(idx), self.rotate(idx)) {
            (None, None, None, None, None) => None,
            (None, None, dx, dy, phi) => Some(Move {
                abs_x: None,
                abs_y: None,
                rel: rel(dx, dy),
                rot: phi
            }),
            (x, y, dx, dy, phi) => Some(Move {
                abs_x: x.map(|l| l.resolve(options)),
                abs_y: y.map(|l| l.resolve(options)),
                rel: rel(dx, dy),
                rot: phi
            })
        }
    }
}

struct Move {
    abs_x: Option<f32>,
    abs_y: Option<f32>,
    rel: Vector2F,
    rot: Option<f32>
}