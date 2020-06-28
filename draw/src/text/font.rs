
use font::{Font, GlyphId};
use std::sync::Arc;
use pathfinder_content::{
    outline::{Outline},
};
use pathfinder_geometry::{
    vector::Vector2F,
    transform2d::{Transform2F},
};
pub type FontArc = Arc<dyn Font + Sync + Send>;

lazy_static! {
    static ref LATIN_MODERN: FontArc = Arc::from(font::parse(include_bytes!("../../../resources/latinmodern-math.otf")));
}

pub struct FontCollection {
    fonts: Vec<FontArc>
}
impl FontCollection {
    pub fn new() -> FontCollection {
        FontCollection { fonts: vec![] }
    }
    pub fn debug() -> FontCollection {
        FontCollection { fonts: vec![LATIN_MODERN.clone()] }
    }
    pub fn add_font(&mut self, font: FontArc) {
        self.fonts.push(font);
    }
    pub fn text(&self, string: &str, transform: Transform2F) -> Outline {
        use std::iter::once;
        let fonts = &*self.fonts;

        let mut gids: Vec<(u32, GlyphId)> = string.chars().flat_map(|c| {
            fonts.iter().enumerate().flat_map(move |(font_idx, font)| font.gid_for_unicode_codepoint(c as u32).map(|gid| (font_idx as u32, gid)))
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
        
        let scale = Transform2F::from_scale(Vector2F::new(1.0, -1.0));
        let mut last_gid = None;
        let mut offset = Vector2F::default();
        let mut outline = Outline::new();
        for &(font_idx, gid) in gids.iter() {
            let face = &fonts[font_idx as usize];
            if let Some(glyph) = face.glyph(gid) {
                if let Some(left) = last_gid.replace(gid) {
                    offset = offset + Vector2F::new(face.kerning(left, gid), 0.0);
                }
                let tr = transform * scale * face.font_matrix();
                let advance = tr * glyph.metrics.advance;

                let glyph_tr = Transform2F::from_translation(offset) * tr;

                for contour in glyph.path.contours() {
                    outline.push_contour(contour.clone().transformed(&glyph_tr))
                }

                offset = offset + advance;
            }
        }

        outline
    }
}

enum GlyphData {
    Outline(Outline),
    
}

pub struct Glyph {
    metrics: (),

}

pub struct Layout {

}