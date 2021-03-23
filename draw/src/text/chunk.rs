use unic_bidi::{Level, LevelRun, BidiInfo};
use svg_text::{FontCollection, Layout};
use svg_dom::TextFlow;
use pathfinder_geometry::vector::Vector2F;
use isolang::Language;

/// basic unit of text
pub struct Chunk {
    text: String,
    runs: Vec<(Level, LevelRun)>
}
impl Chunk {
    pub fn new(text: &str, direction: TextFlow) -> Chunk {
        debug!("split {}", text);
        let level = match direction {
            TextFlow::LeftToRight => Level::ltr(),
            TextFlow::RightToLeft => Level::rtl(),
        };
        let bidi_info = BidiInfo::new(text, Some(level));
        let para = &bidi_info.paragraphs[0];
        let line = para.range.clone();
        let (levels, runs) = bidi_info.visual_runs(para, line);
        let runs: Vec<_> = runs.into_iter().map(|run| (levels[run.start], run)).collect();
        for (_, run) in runs.iter() {
            debug!(" - {}", &text[run.clone()]);
        }
        Chunk {
            text: text.into(),
            runs
        }
    }
    pub fn layout(&self, font: &FontCollection, lang: Option<Language>) -> ChunkLayout {
        let mut offset = Vector2F::zero();
        let mut parts = Vec::with_capacity(self.runs.len());
        for (level, run) in self.runs.iter() {
            let layout = font.layout_run(&self.text[run.clone()], level.is_rtl(), lang);

            let advance = layout.metrics.advance;
            let (run_offset, next_offset) = match level.is_rtl() {
                false => (offset, offset + advance),
                true => (offset - advance, offset - advance),
            };
            parts.push((run.start, run_offset, layout));
            offset = next_offset;
        }

        ChunkLayout { parts, advance: offset }
    }
}
pub struct ChunkLayout {
    pub parts: Vec<(usize, Vector2F, Layout)>,
    pub advance: Vector2F,
}