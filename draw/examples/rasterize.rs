use svg_dom::{Svg};
use svg_draw::{DrawSvg};
use svg_text::{FontCollection, Font};
use std::sync::Arc;
use pathfinder_rasterize::Rasterizer;

fn main() {
    env_logger::init();
    let mut args = std::env::args().skip(1);
    let input = args.next().unwrap();
    let data = std::fs::read(input).unwrap();
    let output = args.next().unwrap();

    let fonts = Arc::new(FontCollection::from_fonts(vec![
        Font::load(include_bytes!("../../resources/latinmodern-math.otf")),
        Font::load(include_bytes!("../../resources/NotoNaskhArabic-Regular.ttf")),
        Font::load(include_bytes!("../../resources/NotoSerifBengali-Regular.ttf")),
    ]));

    let svg = Svg::from_data(&data).unwrap();
    let scene = DrawSvg::new(svg, fonts).compose();
    let image = Rasterizer::new().rasterize(scene, None);
    image.save(&output).unwrap();
}
