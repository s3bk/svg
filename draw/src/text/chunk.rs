use unicode_bidi::{Level, LevelRun, BidiInfo};

/// basic unit of text 
/*
 … which means nothing can change within it.
However we can basiically do nothing at parse time, apart from bidi detection.

what can change:
– text-anchor
– font !
– baseline
*/

pub struct Chunk {
    text: String,
    runs: Vec<(Level, LevelRun)>
}
impl Chunk {
    pub fn new(text: &str) -> Chunk {
        let bidi_info = BidiInfo::new(text, None);
        let para = &bidi_info.paragraphs[0];
        let line = para.range.clone();
        let (levels, runs) = bidi_info.visual_runs(para, line);
        let runs = levels.into_iter().zip(runs.into_iter()).collect();
        Chunk {
            text: text.into(),
            runs
        }
    }
}